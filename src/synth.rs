use crate::{
    config::Config,
    engine::{
        engine::Engine,
        event::MidiEvent,
        note::Note,
        sysex::{ManufacturerId, SYSEX_MSG_END, SYSEX_MSG_START},
    },
};
use alsa::{
    Direction::Capture,
    Seq,
    seq::{EvCtrl, EvNote, Event, EventType, PortCap, PortType},
};

use wd_log::{log_debug_ln, log_info_ln, log_panic, log_warn_ln};

#[derive(Debug)]
pub struct Synth {
    client: Seq,
    engine: Engine,
}

impl Synth {
    pub fn new(cfg: &Config) -> Self {
        Self {
            client: Synth::alsa_port_init(),
            engine: Engine::new(cfg),
        }
    }

    fn alsa_port_init() -> Seq {
        // register alsa midi
        let client = match alsa::Seq::open(None, Some(Capture), false) {
            Ok(r) => r,
            Err(err) => {
                log_panic!("{:?}", err);
            }
        };

        match client.client_id() {
            Ok(id) => {
                log_info_ln!("got midi client with id={}", id);
            }
            Err(err) => {
                log_panic!("{:?}", err);
            }
        }

        match client.set_client_name(c"Madaha") {
            Ok(_) => (),
            Err(err) => {
                log_panic!("{:?}", err);
            }
        };

        let port_caps = PortCap::WRITE | PortCap::SUBS_WRITE;
        let port_type = PortType::MIDI_GENERIC
            | PortType::MIDI_GM
            | PortType::MIDI_GM2
            | PortType::MIDI_GS
            | PortType::MIDI_XG
            | PortType::SYNTHESIZER
            | PortType::APPLICATION;
        match client.create_simple_port(c"Madaha MIDI input port", port_caps, port_type) {
            Ok(id) => {
                log_info_ln!("got midi port={}", id);
            }
            Err(err) => {
                log_panic!("{:?}", err);
            }
        };
        client
    }

    pub fn run(&mut self) {
        log_info_ln!("madaha running...");
        // main event loop
        let mut input = self.client.input();
        loop {
            let result = input.event_input();
            match result {
                Ok(ev) => {
                    let event = match self.event_router(ev) {
                        Some(e) => e,
                        None => continue,
                    };
                    self.engine.on_event(event);
                }
                Err(err) => {
                    log_warn_ln!("{:?}", err);
                }
            }
        }
    }

    fn event_router(&self, ev: Event) -> Option<MidiEvent> {
        let event = match ev.get_type() {
            EventType::Noteon => {
                let note: EvNote = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::NoteOn {
                    channel: note.channel,
                    note: Note::try_from(note.note).unwrap(),
                    velocity: note.velocity,
                    duration: note.duration,
                    off_velocity: note.off_velocity,
                }
            }
            EventType::Noteoff => {
                let note: EvNote = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::NoteOff {
                    channel: note.channel,
                    note: Note::try_from(note.note).unwrap(),
                    velocity: note.velocity,
                    duration: note.duration,
                    off_velocity: note.off_velocity,
                }
            }
            EventType::Controller => {
                let cont: EvCtrl = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::ControlChange {
                    channel: cont.channel,
                    controller: cont.param as u8,
                    value: cont.value as u8,
                }
            }
            EventType::Pgmchange => {
                let pgm: EvCtrl = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::ProgramChange {
                    channel: pgm.channel,
                    program: pgm.value as u8,
                }
            }
            EventType::Pitchbend => {
                let pitch: EvCtrl = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::PitchBend {
                    channel: pitch.channel,
                    value: pitch.value as u16,
                }
            }
            EventType::Regparam => {
                let rpn: EvCtrl = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::RPN {
                    channel: rpn.channel,
                    parameter: rpn.param as u16,
                    value: rpn.value as u16,
                }
            }
            EventType::Nonregparam => {
                let nrpn: EvCtrl = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::NRPN {
                    channel: nrpn.channel,
                    parameter: nrpn.param as u16,
                    value: nrpn.value as u16,
                }
            }
            EventType::Chanpress => {
                let cp: EvCtrl = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::ChannelPressure {
                    channel: cp.channel,
                    pressure: cp.value as u8,
                }
            }
            EventType::Keypress => {
                let kp: EvNote = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::PolyPressure {
                    channel: kp.channel,
                    note: Note::try_from(kp.note).unwrap(),
                    pressure: kp.velocity,
                }
            }
            EventType::Sysex => {
                let sysex = match ev.get_ext() {
                    Some(r) => r,
                    None => return None,
                };
                if sysex.len() <= 2 {
                    log_warn_ln!("found empty valid sysex message");
                    return None;
                }
                if let Some(f) = sysex.first()
                    && *f != SYSEX_MSG_START
                {
                    log_warn_ln!("found bad sysex message, start-byte={:?}", *f);
                    return None;
                }
                if let Some(l) = sysex.last()
                    && *l != SYSEX_MSG_END
                {
                    log_warn_ln!("found bad sysex message, end-byte={:?}", *l);
                    return None;
                }

                // unwarp here no harm, full checked before.
                let data = sysex.get(1..sysex.len().saturating_sub(1)).unwrap();
                MidiEvent::SysEx {
                    manufacturer_id: ManufacturerId::try_from(data[0]).unwrap(),
                    data: data.get(1..).unwrap().into(),
                }
            }
            EventType::TuneRequest => MidiEvent::TuneRequest,
            EventType::Qframe => {
                let qf: EvCtrl = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::MTCQuarterFrame {
                    frame_type: qf.param as u8,
                    value: qf.value as u8,
                }
            }
            EventType::Songpos => {
                let sp: EvCtrl = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::SongPosition {
                    position: (sp.param << 7 | sp.value as u32) as u16,
                }
            }
            EventType::Songsel => {
                let ss: EvCtrl = match ev.get_data() {
                    Some(r) => r,
                    None => return None,
                };
                MidiEvent::SongSelect {
                    song: ss.value as u8,
                }
            }
            EventType::Clock => MidiEvent::TimingClock,
            EventType::Start => MidiEvent::Start,
            EventType::Continue => MidiEvent::Continue,
            EventType::Stop => MidiEvent::Stop,
            EventType::Sensing => MidiEvent::ActiveSensing,
            EventType::Reset => MidiEvent::SystemReset,

            _ => {
                log_debug_ln!("got unused event={:?}", ev);
                return None;
            }
        };

        Some(event)
    }
}

use crate::{
    config::Config,
    midi::{
        consts::NOTE_A4,
        event::MidiEvent,
        sysex::{ManufacturerId, SYSEX_MSG_END, SYSEX_MSG_START},
    },
};
use alsa::{
    Direction::Capture,
    Seq,
    seq::{EvCtrl, EvNote, EventType, PortCap, PortType},
};
use wd_log::{log_debug_ln, log_info_ln, log_panic, log_warn_ln};

#[derive(Debug)]
pub struct Synth {
    client: Seq,
    freq_table: [f64; 128],
}

impl Synth {
    pub fn new(cfg: &Config) -> Self {
        Self {
            client: Synth::alsa_port_init(),
            freq_table: Synth::freq_table_init(cfg),
        }
    }

    fn freq_table_init(cfg: &Config) -> [f64; 128] {
        let semi_tone: f64 = 2f64.powf(1.0 / 12.0);
        let mut freq_table: [f64; 128] = [0.0; 128];
        // caculate
        let mut note: i8 = 0; // Note = C1
        loop {
            let delta = note - NOTE_A4 as i8;
            freq_table[note as usize] = cfg.audio.master_tune * semi_tone.powf(delta as f64);
            if note == 127 {
                break;
            } else {
                note += 1;
            }
        }
        log_debug_ln!("freq_table={:?}", freq_table);
        freq_table
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

    pub fn run(&self) {
        log_info_ln!("madaha running...");
        let mut input = self.client.input();
        // main event loop
        loop {
            match input.event_input() {
                Ok(ev) => {
                    let event = match ev.get_type() {
                        EventType::Noteon => {
                            let note: EvNote = match ev.get_data() {
                                Some(r) => r,
                                None => continue,
                            };
                            MidiEvent::NoteOn {
                                channel: note.channel,
                                note: note.note,
                                velocity: note.velocity,
                                duration: note.duration,
                                off_velocity: note.off_velocity,
                            }
                        }
                        EventType::Noteoff => {
                            let note: EvNote = match ev.get_data() {
                                Some(r) => r,
                                None => continue,
                            };
                            MidiEvent::NoteOff {
                                channel: note.channel,
                                note: note.note,
                                velocity: note.velocity,
                                duration: note.duration,
                                off_velocity: note.off_velocity,
                            }
                        }
                        EventType::Controller => {
                            let cont: EvCtrl = match ev.get_data() {
                                Some(r) => r,
                                None => continue,
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
                                None => continue,
                            };
                            MidiEvent::ProgramChange {
                                channel: pgm.channel,
                                program: pgm.value as u8,
                            }
                        }
                        EventType::Pitchbend => {
                            let pitch: EvCtrl = match ev.get_data() {
                                Some(r) => r,
                                None => continue,
                            };
                            MidiEvent::PitchBend {
                                channel: pitch.channel,
                                value: pitch.value as u16,
                            }
                        }
                        EventType::Regparam => {
                            let rpn: EvCtrl = match ev.get_data() {
                                Some(r) => r,
                                None => continue,
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
                                None => continue,
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
                                None => continue,
                            };
                            MidiEvent::ChannelPressure {
                                channel: cp.channel,
                                pressure: cp.value as u8,
                            }
                        }
                        EventType::Keypress => {
                            let kp: EvNote = match ev.get_data() {
                                Some(r) => r,
                                None => continue,
                            };
                            MidiEvent::PolyPressure {
                                channel: kp.channel,
                                note: kp.note,
                                pressure: kp.velocity,
                            }
                        }
                        EventType::Sysex => {
                            let sysex = match ev.get_ext() {
                                Some(r) => r,
                                None => continue,
                            };
                            if sysex.len() <= 2 {
                                log_warn_ln!("found empty valid sysex message");
                                continue;
                            }
                            if let Some(f) = sysex.first()
                                && *f != SYSEX_MSG_START
                            {
                                log_warn_ln!("found bad sysex message, start-byte={:?}", *f);
                                continue;
                            }
                            if let Some(l) = sysex.last()
                                && *l != SYSEX_MSG_END
                            {
                                log_warn_ln!("found bad sysex message, end-byte={:?}", *l);
                                continue;
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
                                None => continue,
                            };
                            MidiEvent::MTCQuarterFrame {
                                frame_type: qf.param as u8,
                                value: qf.value as u8,
                            }
                        }
                        EventType::Songpos => {
                            let sp: EvCtrl = match ev.get_data() {
                                Some(r) => r,
                                None => continue,
                            };
                            MidiEvent::SongPosition {
                                position: (sp.param << 7 | sp.value as u32) as u16,
                            }
                        }
                        EventType::Songsel => {
                            let ss: EvCtrl = match ev.get_data() {
                                Some(r) => r,
                                None => continue,
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
                            continue;
                        }
                    };
                    log_debug_ln!("got midi event {:?}", event);
                }
                Err(err) => {
                    log_warn_ln!("{:?}", err);
                }
            }
        }
    }
}

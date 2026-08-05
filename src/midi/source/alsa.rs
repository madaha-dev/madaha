//! ALSA Seq MIDI input (the classic backend; creates a MIDI input port)
use alsa::seq::{Event, EventType, PortCap, PortType, Seq};
use alsa::Direction;
use wd_log::{log_info_ln, log_panic, log_warn_ln};

use crate::midi::event::MidiEvent;
use crate::midi::sysex::{ManufacturerId, SYSEX_MSG_END, SYSEX_MSG_START};

use super::MidiSource;

pub struct AlsaMidiSource {
    seq: Seq,
}

impl AlsaMidiSource {
    pub fn open() -> Result<Self, String> {
        let client = match Seq::open(None, Some(Direction::Capture), false) {
            Ok(r) => r,
            Err(err) => return Err(format!("alsa seq open: {err}")),
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
        }

        Ok(Self { seq: client })
    }

    fn event_router(&self, ev: Event) -> Option<MidiEvent> {
        use EventType::*;
        let event = match ev.get_type() {
            Noteon => {
                let note: alsa::seq::EvNote = ev.get_data()?;
                MidiEvent::NoteOn {
                    channel: note.channel,
                    note: crate::midi::note::Note::try_from(note.note).ok()?,
                    velocity: note.velocity,
                    duration: note.duration,
                    off_velocity: note.off_velocity,
                }
            }
            Noteoff => {
                let note: alsa::seq::EvNote = ev.get_data()?;
                MidiEvent::NoteOff {
                    channel: note.channel,
                    note: crate::midi::note::Note::try_from(note.note).ok()?,
                    velocity: note.velocity,
                    duration: note.duration,
                    off_velocity: note.off_velocity,
                }
            }
            Controller => {
                let cont: alsa::seq::EvCtrl = ev.get_data()?;
                MidiEvent::ControlChange {
                    channel: cont.channel,
                    controller: cont.param as u8,
                    value: cont.value as u8,
                }
            }
            Pgmchange => {
                let pgm: alsa::seq::EvCtrl = ev.get_data()?;
                MidiEvent::ProgramChange {
                    channel: pgm.channel,
                    program: pgm.value as u8,
                }
            }
            Pitchbend => {
                let pitch: alsa::seq::EvCtrl = ev.get_data()?;
                MidiEvent::PitchBend {
                    channel: pitch.channel,
                    value: pitch.value as u16,
                }
            }
            Regparam => {
                let rpn: alsa::seq::EvCtrl = ev.get_data()?;
                MidiEvent::RPN {
                    channel: rpn.channel,
                    parameter: rpn.param as u16,
                    value: (rpn.value as u16) << 7,
                }
            }
            Nonregparam => {
                let nrpn: alsa::seq::EvCtrl = ev.get_data()?;
                MidiEvent::NRPN {
                    channel: nrpn.channel,
                    parameter: nrpn.param as u16,
                    value: (nrpn.value as u16) << 7,
                }
            }
            Chanpress => {
                let cp: alsa::seq::EvCtrl = ev.get_data()?;
                MidiEvent::ChannelPressure {
                    channel: cp.channel,
                    pressure: cp.value as u8,
                }
            }
            Keypress => {
                let kp: alsa::seq::EvNote = ev.get_data()?;
                MidiEvent::PolyPressure {
                    channel: kp.channel,
                    note: crate::midi::note::Note::try_from(kp.note).unwrap(),
                    pressure: kp.velocity,
                }
            }
            Sysex => {
                let sysex = ev.get_ext()?;
                if sysex.len() <= 2 {
                    log_warn_ln!("found empty valid sysex message");
                    return Option::None;
                }
                if let Some(f) = sysex.first()
                    && *f != SYSEX_MSG_START
                {
                    log_warn_ln!("found bad sysex message, start-byte={:?}", *f);
                    return Option::None;
                }
                if let Some(l) = sysex.last()
                    && *l != SYSEX_MSG_END
                {
                    log_warn_ln!("found bad sysex message, end-byte={:?}", *l);
                    return Option::None;
                }

                let data = sysex.get(1..sysex.len().saturating_sub(1))?;
                MidiEvent::SysEx {
                    manufacturer_id: ManufacturerId::try_from(data[0]).ok()?,
                    data: data.get(1..)?.into(),
                }
            }
            TuneRequest => MidiEvent::TuneRequest,
            Qframe => {
                let qf: alsa::seq::EvCtrl = ev.get_data()?;
                MidiEvent::MTCQuarterFrame {
                    frame_type: qf.param as u8,
                    value: qf.value as u8,
                }
            }
            Songpos => {
                let sp: alsa::seq::EvCtrl = ev.get_data()?;
                MidiEvent::SongPosition {
                    position: (sp.param << 7 | sp.value as u32) as u16,
                }
            }
            Songsel => {
                let ss: alsa::seq::EvCtrl = ev.get_data()?;
                MidiEvent::SongSelect {
                    song: ss.value as u8,
                }
            }
            Clock => MidiEvent::TimingClock,
            Start => MidiEvent::Start,
            Continue => MidiEvent::Continue,
            Stop => MidiEvent::Stop,
            Sensing => MidiEvent::ActiveSensing,
            Reset => MidiEvent::SystemReset,

            _ => {
                return Option::None;
            }
        };
        Some(event)
    }
}

impl MidiSource for AlsaMidiSource {
    fn next_event(&mut self) -> Option<MidiEvent> {
        loop {
            let mut input = self.seq.input();
            let result = input.event_input();
            match result {
                Ok(ev) => {
                    if let Some(event) = self.event_router(ev) {
                        return Some(event);
                    }
                }
                Err(err) => {
                    log_warn_ln!("alsa midi: {err:?}");
                    return Option::None;
                }
            }
        }
    }
}

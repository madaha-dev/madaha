use alsa::{
    Direction::Capture,
    Seq,
    seq::{EvCtrl, EvNote, Event, EventType, PortCap, PortType},
};
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

use crate::audio::AudioRender;
use crate::audio::AudioRenderActions;
use crate::{
    config::Config,
    midi::{
        engine::Engine,
        event::MidiEvent,
        note::Note,
        sysex::{ManufacturerId, SYSEX_MSG_END, SYSEX_MSG_START},
    },
};

use wd_log::{log_debug_ln, log_info_ln, log_panic, log_warn_ln};

#[derive(Debug)]
pub struct Synth {
    client: Seq,
}

impl Synth {
    pub fn new() -> Self {
        Self {
            client: Synth::alsa_port_init(),
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

    fn run_audio(&self, cfg: &Config, rx: Receiver<AudioRenderActions>) {
        let source_sample_rate = cfg.sound_module.module_type.get_sample_rate();
        let target_sample_rate = cfg.audio.sample_rate as f32;
        let max_polyphony = cfg.midi.max_polyphony;
        let count = (max_polyphony * cfg.midi.poly_replicant / 100) as usize;
        let scoring = cfg.midi.scoring.clone();

        thread::spawn(move || {
            log_info_ln!("audio thread running...");
            let mut audio_render = AudioRender::new(
                count,
                max_polyphony,
                source_sample_rate,
                target_sample_rate,
                scoring,
                rx,
            );
            loop {
                audio_render.audio_render();
            }
        });
    }

    pub fn run(&mut self, cfg: &Config) {
        log_info_ln!("madaha running...");
        // main event loop
        let (tx, rx) = sync_channel(cfg.midi.channel_size);
        self.run_audio(cfg, rx);
        
        let mut engine = Engine::new(cfg, tx);
        let mut input = self.client.input();
        loop {
            let result = input.event_input();
            match result {
                Ok(ev) => {
                    let event = match self.event_router(ev) {
                        Some(e) => e,
                        None => continue,
                    };
                    engine.on_event(event);
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
                let note: EvNote = ev.get_data()?;
                MidiEvent::NoteOn {
                    channel: note.channel,
                    note: Note::try_from(note.note).ok()?,
                    velocity: note.velocity,
                    duration: note.duration,
                    off_velocity: note.off_velocity,
                }
            }
            EventType::Noteoff => {
                let note: EvNote = ev.get_data()?;
                MidiEvent::NoteOff {
                    channel: note.channel,
                    note: Note::try_from(note.note).ok()?,
                    velocity: note.velocity,
                    duration: note.duration,
                    off_velocity: note.off_velocity,
                }
            }
            EventType::Controller => {
                let cont: EvCtrl = ev.get_data()?;
                MidiEvent::ControlChange {
                    channel: cont.channel,
                    controller: cont.param as u8,
                    value: cont.value as u8,
                }
            }
            EventType::Pgmchange => {
                let pgm: EvCtrl = ev.get_data()?;
                MidiEvent::ProgramChange {
                    channel: pgm.channel,
                    program: pgm.value as u8,
                }
            }
            EventType::Pitchbend => {
                let pitch: EvCtrl = ev.get_data()?;
                MidiEvent::PitchBend {
                    channel: pitch.channel,
                    value: pitch.value as u16,
                }
            }
            EventType::Regparam => {
                let rpn: EvCtrl = ev.get_data()?;
                MidiEvent::RPN {
                    channel: rpn.channel,
                    parameter: rpn.param as u16,
                    value: (rpn.value as u16) << 7,
                }
            }
            EventType::Nonregparam => {
                let nrpn: EvCtrl = ev.get_data()?;
                MidiEvent::NRPN {
                    channel: nrpn.channel,
                    parameter: nrpn.param as u16,
                    value: (nrpn.value as u16) << 7,
                }
            }
            EventType::Chanpress => {
                let cp: EvCtrl = ev.get_data()?;
                MidiEvent::ChannelPressure {
                    channel: cp.channel,
                    pressure: cp.value as u8,
                }
            }
            EventType::Keypress => {
                let kp: EvNote = ev.get_data()?;
                MidiEvent::PolyPressure {
                    channel: kp.channel,
                    note: Note::try_from(kp.note).unwrap(),
                    pressure: kp.velocity,
                }
            }
            EventType::Sysex => {
                let sysex = ev.get_ext()?;
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
                let data = sysex.get(1..sysex.len().saturating_sub(1))?;
                MidiEvent::SysEx {
                    manufacturer_id: ManufacturerId::try_from(data[0]).ok()?,
                    data: data.get(1..)?.into(),
                }
            }
            EventType::TuneRequest => MidiEvent::TuneRequest,
            EventType::Qframe => {
                let qf: EvCtrl = ev.get_data()?;
                MidiEvent::MTCQuarterFrame {
                    frame_type: qf.param as u8,
                    value: qf.value as u8,
                }
            }
            EventType::Songpos => {
                let sp: EvCtrl = ev.get_data()?;
                MidiEvent::SongPosition {
                    position: (sp.param << 7 | sp.value as u32) as u16,
                }
            }
            EventType::Songsel => {
                let ss: EvCtrl = ev.get_data()?;
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

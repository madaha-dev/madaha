//! PipeWire MIDI input: a MIDI input stream whose process callback decodes
//! raw MIDI bytes and pushes events into a channel
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use pipewire as pw;

use crate::midi::event::MidiEvent;

use super::parse_midi_bytes;
use super::MidiSource;

pub struct PipewireMidiSource {
    _handle: Option<std::thread::JoinHandle<()>>,
    rx: Receiver<MidiEvent>,
}

impl PipewireMidiSource {
    pub fn open() -> Result<Self, String> {
        let (tx, rx): (SyncSender<MidiEvent>, Receiver<MidiEvent>) = sync_channel(1024);

        let handle = std::thread::Builder::new()
            .name("pipewire-midi".into())
            .spawn(move || {
                if let Err(e) = Self::run_mainloop(tx) {
                    eprintln!("pipewire midi mainloop error: {e}");
                }
            })
            .map_err(|e| format!("pipewire thread: {e}"))?;

        Ok(Self {
            _handle: Some(handle),
            rx,
        })
    }

    fn run_mainloop(tx: SyncSender<MidiEvent>) -> Result<(), pw::Error> {
        pw::init();
        let mainloop = pw::main_loop::MainLoopRc::new(None)?;
        let context = pw::context::ContextRc::new(&mainloop, None)?;
        let core = context.connect_rc(None)?;

        let props = pw::properties::properties! {
            *pw::keys::MEDIA_TYPE => "Midi",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Music",
        };
        let stream = pw::stream::StreamRc::new(core, "madaha-midi", props)?;

        let _listener = stream
            .add_local_listener_with_user_data(())
            .process(move |stream, _: &mut ()| {
                if let Some(mut buffer) = stream.dequeue_buffer() {
                    let datas = buffer.datas_mut();
                    if let Some(data) = datas.first_mut() {
                        let size = data.chunk().size() as usize;
                        if size > 0 {
                            if let Some(bytes) = data.data() {
                                let mut running = None;
                                let mut sx = Vec::new();
                                let mut events = Vec::new();
                                parse_midi_bytes(&bytes[..size.min(bytes.len())], &mut running, &mut sx, &mut events);
                                for ev in events {
                                    let _ = tx.try_send(ev);
                                }
                            }
                        }
                    }
                }
            })
            .register()?;

        stream.connect(
            pw::spa::utils::Direction::Input,
            None,
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut [],
        )?;

        mainloop.run();
        Ok(())
    }
}

impl MidiSource for PipewireMidiSource {
    fn next_event(&mut self) -> Option<MidiEvent> {
        self.rx.recv().ok()
    }
}

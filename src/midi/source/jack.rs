//! Jack MIDI input: a MIDI input port whose process callback decodes raw
//! MIDI bytes and pushes events into a channel
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use jack::contrib::ClosureProcessHandler;
use jack::{Client, ClientOptions, Control, MidiIn, Port, PortFlags, ProcessScope};

use crate::midi::event::MidiEvent;

use super::parse_midi_bytes;
use super::MidiSource;

pub struct JackMidiSource {
    _active: jack::AsyncClient<(), ClosureProcessHandler<(), Box<dyn FnMut(&Client, &ProcessScope) -> Control + Send>>>,
    rx: Receiver<MidiEvent>,
}

impl JackMidiSource {
    pub fn open() -> Result<Self, String> {
        let (client, _status) =
            Client::new("madaha-midi", ClientOptions::NO_START_SERVER)
                .map_err(|e| format!("jack open: {e}"))?;
        let port: Port<MidiIn> = client
            .register_port("midi_in", MidiIn::default())
            .map_err(|e| format!("jack midi port: {e}"))?;
        let _ = &PortFlags::IS_INPUT;

        let (tx, rx): (SyncSender<MidiEvent>, Receiver<MidiEvent>) = sync_channel(1024);

        let process: ClosureProcessHandler<
            (),
            Box<dyn FnMut(&Client, &ProcessScope) -> Control + Send>,
        > = ClosureProcessHandler::new(Box::new(move |_client: &Client, ps: &ProcessScope| {
            let mut running = None;
            let mut sx = Vec::new();
            let mut events = Vec::new();
            for raw in port.iter(ps) {
                parse_midi_bytes(raw.bytes, &mut running, &mut sx, &mut events);
            }
            for ev in events {
                let _ = tx.try_send(ev);
            }
            Control::Continue
        }));

        let active = jack::AsyncClient::new(client, (), process)
            .map_err(|e| format!("jack activate: {e}"))?;

        Ok(Self { _active: active, rx })
    }
}

impl MidiSource for JackMidiSource {
    fn next_event(&mut self) -> Option<MidiEvent> {
        self.rx.recv().ok()
    }
}

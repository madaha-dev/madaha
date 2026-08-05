//! MIDI input sources: ALSA Seq / Jack MIDI / PipeWire MIDI
//!
//! `MidiSource` blocks on `next_event()`; the selected backend feeds it
//! decoded `MidiEvent`s.

use std::sync::mpsc::{Receiver, sync_channel};

use crate::config::MidiInputEngine;
use crate::midi::event::MidiEvent;
use crate::midi::note::Note;

pub mod alsa;
pub mod jack;
pub mod pipewire;

pub trait MidiSource: Send {
    /// Block until the next MIDI event arrives
    fn next_event(&mut self) -> Option<MidiEvent>;
}

/// Create the MIDI input source selected by `cfg.midi.input_engine`
pub fn create_midi_source(
    engine: MidiInputEngine,
) -> Result<Box<dyn MidiSource>, String> {
    match engine {
        MidiInputEngine::Alsa => alsa::AlsaMidiSource::open().map(|s| Box::new(s) as Box<dyn MidiSource>),
        MidiInputEngine::Jack => jack::JackMidiSource::open().map(|s| Box::new(s) as Box<dyn MidiSource>),
        MidiInputEngine::Pipewire => {
            pipewire::PipewireMidiSource::open().map(|s| Box::new(s) as Box<dyn MidiSource>)
        }
    }
}

/// Shared event channel (backends push, `next_event` pops)
pub fn event_channel() -> (std::sync::mpsc::SyncSender<MidiEvent>, Receiver<MidiEvent>) {
    sync_channel(1024)
}

/// Parse raw MIDI bytes into events (handles running status + sysex spanning)
///
/// The parser keeps `running_status` across calls so multi-byte messages
/// split at buffer boundaries still decode correctly. Sysex is accumulated
/// until F7.
pub fn parse_midi_bytes(
    bytes: &[u8],
    running_status: &mut Option<u8>,
    sysex_accum: &mut Vec<u8>,
    out: &mut Vec<MidiEvent>,
) {
    let mut i = 0;
    let mut status: Option<u8> = *running_status;
    while i < bytes.len() {
        let b = bytes[i];
        if b & 0x80 != 0 {
            // ── Status byte ──
            if b == 0xF0 {
                // SysEx start
                sysex_accum.clear();
                sysex_accum.push(b);
                i += 1;
                continue;
            }
            if b == 0xF7 {
                // SysEx end
                sysex_accum.push(b);
                if let Some(ev) = parse_sysex(std::mem::take(sysex_accum)) {
                    out.push(ev);
                }
                sysex_accum.clear();
                i += 1;
                continue;
            }
            if !sysex_accum.is_empty() {
                // Status byte inside sysex payload (rare)
                sysex_accum.push(b);
                i += 1;
                continue;
            }
            status = Some(b);
            if !is_channel_status(b) {
                if let Some(ev) = parse_system(b) {
                    out.push(ev);
                }
                status = None;
                i += 1;
                continue;
            }
            i += 1;
        } else if !sysex_accum.is_empty() {
            // ── Sysex payload byte (no status needed) ──
            sysex_accum.push(b);
            i += 1;
            continue;
        }
        // ── Data byte: decode with (running) status ──
        let Some(s) = status else {
            i += 1;
            continue;
        };
        let data_len = match s & 0xF0 {
            0xC0 | 0xD0 => 1,
            _ => 2,
        };
        if i + data_len > bytes.len() {
            break; // incomplete; wait for more
        }
        let d1 = bytes[i];
        let d2 = if data_len == 2 { bytes[i + 1] } else { 0 };
        let channel = s & 0x0F;
        let kind = s & 0xF0;
        let note = Note::try_from(d1 & 0x7F).ok();
        let ev = match kind {
            0x80 => note.map(|n| MidiEvent::NoteOff {
                channel,
                note: n,
                velocity: d2 & 0x7F,
                off_velocity: 0,
                duration: 0,
            }),
            0x90 => note.map(|n| MidiEvent::NoteOn {
                channel,
                note: n,
                velocity: d2 & 0x7F,
                off_velocity: 0,
                duration: 0,
            }),
            0xA0 => note.map(|n| MidiEvent::PolyPressure {
                channel,
                note: n,
                pressure: d2 & 0x7F,
            }),
            0xB0 => Some(MidiEvent::ControlChange {
                channel,
                controller: d1 & 0x7F,
                value: d2 & 0x7F,
            }),
            0xC0 => Some(MidiEvent::ProgramChange {
                channel,
                program: d1 & 0x7F,
            }),
            0xD0 => Some(MidiEvent::ChannelPressure {
                channel,
                pressure: d1 & 0x7F,
            }),
            0xE0 => Some(MidiEvent::PitchBend {
                channel,
                value: ((d2 as u16) << 7) | (d1 as u16),
            }),
            _ => None,
        };
        if let Some(ev) = ev {
            out.push(ev);
        }
        i += data_len;
    }
    *running_status = if sysex_accum.is_empty() {
        status.filter(|&b| is_channel_status(b))
    } else {
        *running_status
    };
}

fn is_channel_status(b: u8) -> bool {
    (0x80..=0xEF).contains(&b)
}

fn parse_system(b: u8) -> Option<MidiEvent> {
    match b {
        0xF1 => Some(MidiEvent::MTCQuarterFrame {
            frame_type: 0,
            value: 0,
        }),
        0xF2 => Some(MidiEvent::SongPosition { position: 0 }),
        0xF3 => Some(MidiEvent::SongSelect { song: 0 }),
        0xF4 => Some(MidiEvent::TuneRequest),
        0xF8 => Some(MidiEvent::TimingClock),
        0xFA => Some(MidiEvent::Start),
        0xFB => Some(MidiEvent::Continue),
        0xFC => Some(MidiEvent::Stop),
        0xFE => Some(MidiEvent::ActiveSensing),
        0xFF => Some(MidiEvent::SystemReset),
        _ => None,
    }
}

fn parse_sysex(accum: Vec<u8>) -> Option<MidiEvent> {
    // accum = [F0, ... , F7]
    if accum.len() <= 2 {
        return None;
    }
    let data = &accum[1..accum.len() - 1];
    let manufacturer_id = data.first().copied()?;
    let mfid = crate::midi::sysex::ManufacturerId::try_from(manufacturer_id).ok()?;
    Some(MidiEvent::SysEx {
        manufacturer_id: mfid,
        data: data.get(1..)?.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(bytes: &[u8]) -> Vec<MidiEvent> {
        let mut rs = None;
        let mut sx = Vec::new();
        let mut out = Vec::new();
        parse_midi_bytes(bytes, &mut rs, &mut sx, &mut out);
        out
    }

    #[test]
    fn note_on_off() {
        let evs = parse(&[0x90, 60, 100, 0x80, 60, 0]);
        assert_eq!(evs.len(), 2);
        assert!(matches!(evs[0], MidiEvent::NoteOn { channel: 0, velocity: 100, .. }));
        assert!(matches!(evs[1], MidiEvent::NoteOff { .. }));
    }

    #[test]
    fn running_status() {
        // 0x90 running: NoteOn 60/100, NoteOn 64/90 (no status byte)
        let evs = parse(&[0x90, 60, 100, 64, 90]);
        assert_eq!(evs.len(), 2);
        assert!(matches!(
            evs[1],
            MidiEvent::NoteOn { note: crate::midi::note::Note::E3, velocity: 90, .. }
        ));
    }

    #[test]
    fn sysex_split_across_buffers() {
        let mut rs = None;
        let mut sx = Vec::new();
        let mut out = Vec::new();
        parse_midi_bytes(&[0xF0, 0x7E, 0x09, 0x03], &mut rs, &mut sx, &mut out);
        assert!(out.is_empty(), "sysex incomplete");
        parse_midi_bytes(&[0xF7], &mut rs, &mut sx, &mut out);
        assert_eq!(out.len(), 1);
        assert!(matches!(&out[0], MidiEvent::SysEx { .. }));
    }

    #[test]
    fn pitch_bend_14bit() {
        let evs = parse(&[0xE0, 0x00, 0x40]); // 0x2000 = center
        assert!(matches!(evs[0], MidiEvent::PitchBend { value: 0x2000, .. }));
    }
}

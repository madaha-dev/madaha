//! Active Sensing (0xFE) heartbeat watchdog
//!
//! MIDI spec: the transmitter sends Active Sensing within 300ms intervals;
//! if no byte arrives for longer than the timeout, the connection is
//! considered broken and the receiver resets (all notes off + controller reset).
//!
//! Design:
//! - `beat()` is called passively on each received 0xFE (we never send it)
//! - a watchdog thread checks the last-beat timestamp; on timeout it
//!   deactivates the client and performs the reset itself (all parts'
//!   controllers + pitchbend reset, and audio ReleaseAll), so the reset
//!   happens automatically even with no further MIDI traffic.
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::audio::AudioRenderActions;
use crate::midi::consts::PITCH_BEND_MIDDLE;
use crate::midi::part::DataEntrySelect;
use crate::double_buffer::DoubleBuffered;
use crate::midi::Part;

/// Wall-clock milliseconds
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug)]
pub struct ActiveSensingState {
    /// Client connected (a heartbeat was received)
    active: AtomicBool,
    /// Timestamp (epoch ms) of the last received heartbeat
    last_beat_ms: AtomicU64,
    /// Timeout before the connection is considered broken
    timeout_ms: u64,
    /// Watchdog tick interval
    tick_ms: u64,
}

impl ActiveSensingState {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            active: AtomicBool::new(false),
            last_beat_ms: AtomicU64::new(0),
            timeout_ms,
            tick_ms: 100,
        }
    }

    /// Record a heartbeat (passive; this type never sends Active Sensing)
    pub fn beat(&self) {
        self.active.store(true, Ordering::Relaxed);
        self.last_beat_ms.store(now_ms(), Ordering::Relaxed);
    }

    /// Client currently active (heartbeat within timeout)
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Watchdog thread: on timeout, deactivate + reset all parts + release audio
    pub fn spawn_watchdog(
        self: &Arc<Self>,
        parts: Vec<Arc<DoubleBuffered<Part>>>,
        tx: std::sync::mpsc::SyncSender<AudioRenderActions>,
    ) {
        let state = self.clone();
        std::thread::Builder::new()
            .name("active-sensing-watchdog".into())
            .spawn(move || loop {
                std::thread::sleep(Duration::from_millis(state.tick_ms));
                if !state.active.load(Ordering::Relaxed) {
                    continue;
                }
                let elapsed = now_ms().saturating_sub(state.last_beat_ms.load(Ordering::Relaxed));
                if elapsed <= state.timeout_ms {
                    continue;
                }
                // Connection lost: deactivate + reset
                state.active.store(false, Ordering::Relaxed);
                for part in &parts {
                    part.write_with(|p| {
                        // MIDI 1.0 timeout behavior: all notes off + reset controllers
                        p.controller.reset();
                        p.rpn.reset(); // RPN state (bend sensitivity, tuning selects)
                        p.pitchbend = PITCH_BEND_MIDDLE;
                        p.cat_value = 0;
                        p.pat_values = [0; 0x80];
                        p.last_note = None; // portamento reference
                        p.data_entry_select = DataEntrySelect::None;
                    });
                    // Commit immediately so the reset is visible without further MIDI traffic
                    part.swap();
                    let _ = tx.send(AudioRenderActions::ReleaseAll { part: part.clone() });
                }
            })
            .expect("failed to spawn active-sensing watchdog");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_activates_and_timeout_deactivates() {
        let state = Arc::new(ActiveSensingState::new(500));
        assert!(!state.is_active());
        state.beat();
        assert!(state.is_active());

        // Simulate a stale heartbeat (timeout elapsed)
        state
            .last_beat_ms
            .store(now_ms().saturating_sub(1000), Ordering::Relaxed);

        // Watchdog without parts: deactivates after a tick
        state.spawn_watchdog(vec![], std::sync::mpsc::sync_channel(16).0);
        std::thread::sleep(Duration::from_millis(250));
        assert!(!state.is_active(), "watchdog must deactivate on timeout");
    }

    #[test]
    fn never_beat_stays_inactive() {
        let state = Arc::new(ActiveSensingState::new(300));
        let (tx, rx) = std::sync::mpsc::sync_channel(8);
        state.spawn_watchdog(vec![], tx);
        // Run longer than the timeout; no heartbeat was ever sent
        std::thread::sleep(Duration::from_millis(700));
        assert!(
            !state.is_active(),
            "without any heartbeat the watchdog must stay inactive"
        );
        assert!(
            rx.try_recv().is_err(),
            "no reset events may fire without a heartbeat"
        );
    }

    #[test]
    fn fresh_heartbeat_keeps_active() {
        let state = Arc::new(ActiveSensingState::new(500));
        state.beat();
        state.spawn_watchdog(vec![], std::sync::mpsc::sync_channel(16).0);
        // Keep beating within the timeout
        for _ in 0..3 {
            std::thread::sleep(Duration::from_millis(150));
            state.beat();
        }
        assert!(state.is_active(), "fresh heartbeats must keep the client active");
    }
}

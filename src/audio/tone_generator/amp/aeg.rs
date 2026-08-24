/// AEG (Amplitude Envelope Generator)
///
/// State machine: Attack → Decay → Sustain → Release → Finished
/// Output: level ∈ [0, 1] used directly as volume gain
///
/// Alignment notes:
/// - Part 08 pp 1A-1C (EG Attack/Decay/Release Time) applies to AEG/FEG/PEG alike
///   (XG sound source model), stage times taken from Part parameters (note-on snapshot)
/// - Sustain level: no independent field in Part 08 pp 18? → fixed 0.7 (approximation),
///   to be aligned exactly once the 2006LE data file is supported
use std::time::Duration;

use crate::audio::interface::Audio;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AEGStage {
    /// Key-on delay: holds the level at 0 for `delay_time` before Attack
    /// (element[72] eg_delay (KeyOnDelay) → 0x500 stage table, S-YXG50 FUN_10018cf0).
    Delay,
    Attack,
    Decay,
    Sustain,
    /// Damper-hold stage: entered from Sustain while the sustain pedal is
    /// held with a damper policy active (S-YXG50 acoustic-piano programs or
    /// 2006LE sustain_mode=2). The level slowly decays to zero — the pedal
    /// holds the note but the string energy fades (real piano behavior).
    Damp,
    Release,
    Finished,
}

/// key_on_delay (element[72], 0-127) → delay length in frames @44100 Hz.
/// Dumped from S-YXG50 (0x500 stage table t1 @0x10018cf0 positive path;
/// t1[38]=448 ≈ 10.2 ms). musicbox: element 1 = 22 → 112 frames (2.5 ms),
/// element 2 = 21 → 104 frames (2.4 ms).
pub static KEY_ON_DELAY_TABLE: [u32; 128] = [
    16, 17, 20, 22, 24, 26, 28, 30, 32, 36, 40, 44, 48, 52, 56, 60, 64, 72, 80, 88, 96, 104, 112,
    120, 128, 144, 160, 176, 192, 208, 224, 240, 256, 288, 320, 352, 384, 416, 448, 480, 512, 576,
    640, 704, 768, 832, 896, 960, 1024, 1152, 1280, 1408, 1536, 1664, 1792, 1920, 2048, 2304,
    2560, 2816, 3072, 3328, 3584, 3840, 4097, 4609, 5121, 5633, 6145, 6657, 7169, 7681, 8194,
    9218, 10242, 11267, 12291, 13315, 14339, 15363, 16388, 18436, 20485, 22534, 24582, 26630,
    28679, 30727, 32776, 40970, 49164, 57358, 65568, 73746, 81940, 90139, 98328, 98328, 131200,
    131200, 163880, 163880, 196800, 196800, 229432, 229432, 229432, 229432, 360800, 360800,
    360800, 360800, 491640, 491640, 491640, 491640, 491640, 491640, 491640, 491640, 1016800,
    1016800, 1016800, 1016800, 1016800, 1016800, 1016800, 1016800,
];

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct AEG {
    pub state: AEGStage,
    /// Current level [0, 1]
    pub level: f32,

    /// EG enable (element[71] eg_enable / [41] eg_amp_en; false → always 1.0)
    pub enabled: bool,

    pub attack_time: Duration,
    pub decay_time: Duration,
    pub sustain_level: f32,
    pub release_time: Duration,
    /// Key-on delay (element[72] eg_delay (KeyOnDelay) → 0x500 table): level holds at 0
    /// for this long before Attack starts.
    pub delay_time: Duration,

    /// Damper-hold stage time (approximation; real piano ~2-5 s). To be
    /// aligned once the 2006LE data file is supported.
    pub damp_time: Duration,
    /// Damper flag: when set, Sustain decays through the Damp stage instead
    /// of freezing at sustain_level.
    damper: bool,
    /// NoteOff arrived during the protected Attack/Decay stages: the envelope
    /// finishes Attack → Decay → Sustain first, then releases (XG short-note
    /// behavior — a very short key press still sounds the full attack+decay,
    /// e.g. musicbox hammer strike + bell body before the release tail).
    pending_release: bool,
    /// Level at the start of the Damp stage (sustain_level when entered)
    damp_start_level: f32,
    /// Level at the start of the Release stage (note_off snapshot)
    release_start_level: f32,

    /// 1/time (per second) — precomputed so the per-block tick only multiplies
    /// (f32 division per tick was slow on this machine)
    inv_attack: f32,
    inv_decay: f32,
    inv_release: f32,
    inv_damp: f32,

    stage_started: Duration,
    elapsed_total: Duration,
}

impl AEG {
    pub fn new() -> Self {
        Self {
            state: AEGStage::Attack,
            level: 0.0,
            enabled: true,
            attack_time: Duration::from_millis(5),
            decay_time: Duration::from_millis(100),
            sustain_level: 0.7,
            release_time: Duration::from_millis(100),
            delay_time: Duration::ZERO,
            damp_time: Duration::from_secs(3),
            damper: false,
            pending_release: false,
            damp_start_level: 0.0,
            release_start_level: 0.0,
            inv_attack: 1.0 / 0.005,
            inv_decay: 1.0 / 0.1,
            inv_release: 1.0 / 0.1,
            inv_damp: 1.0 / 3.0,
            stage_started: Duration::ZERO,
            elapsed_total: Duration::ZERO,
        }
    }

    /// Initialize from Part parameters (snapshot at note-on)
    /// `eg_attack/decay/release`: 08 pp 1A-1C, 64=center.
    /// Attack center = 1ms (XG neutral attack must stay sharp enough to pass
    /// percussion transients — musicbox hammer strike lives in the first ~1ms
    /// of the element sample; a 5ms center faded it to ~18%).
    /// `key_on_delay`: element[72] KeyOnDelay → 0x500 stage table (frames).
    pub fn setup(&mut self, eg_attack: u8, eg_decay: u8, eg_release: u8, key_on_delay: u8) {
        self.state = if key_on_delay == 0 {
            AEGStage::Attack
        } else {
            AEGStage::Delay
        };
        self.level = 0.0;
        self.stage_started = Duration::ZERO;
        self.elapsed_total = Duration::ZERO;
        self.delay_time = delay_frames_to_duration(key_on_delay);
        self.attack_time = Duration::from_millis(param_to_ms(eg_attack, 1.0) as u64);
        self.decay_time = Duration::from_millis(param_to_ms(eg_decay, 100.0) as u64);
        self.release_time = Duration::from_millis(param_to_ms(eg_release, 100.0) as u64);
        self.inv_attack = inv_secs(self.attack_time);
        self.inv_decay = inv_secs(self.decay_time);
        self.inv_release = inv_secs(self.release_time);
        self.sustain_level = 0.7;
        self.damper = false;
        self.pending_release = false;
        self.damp_start_level = 0.0;
        self.release_start_level = 0.0;
    }

    /// Enable/disable the damper-hold stage. While enabled, the envelope
    /// decays through `Damp` instead of freezing at sustain_level.
    pub fn set_damper(&mut self, on: bool) {
        self.damper = on;
    }

    /// 元素级 EG 时间覆写（S-YXG50：曲线 A → EG 段速率，每个元素独立；
    /// madaha 以 log 速率 → 时间近似）。更新 inv_* 缓存。
    pub fn set_element_eg(&mut self, attack: Duration, decay: Duration) {
        self.attack_time = attack;
        self.decay_time = decay;
        self.inv_attack = inv_secs(self.attack_time);
        self.inv_decay = inv_secs(self.decay_time);
    }

    /// 元素级 release 覆写（曲线 A 原始值 → 长余音；yxg50 实测 ~5.6s）
    pub fn set_element_release(&mut self, release: Duration) {
        self.release_time = release;
        self.inv_release = inv_secs(self.release_time);
    }

    /// 元素级 sustain 电平覆写：短采样（打击乐类，如 Marimba）在 decay 后
    /// 冻结在高电平会持续振荡（听感像 EP）。引擎对这些音色的 EG 段 3 目标
    /// 很低（近 0），故短采样 → 低 sustain。
    pub fn set_element_sustain(&mut self, level: f32) {
        self.sustain_level = level;
    }

    pub fn note_off(&mut self) {
        if matches!(self.state, AEGStage::Finished | AEGStage::Release) {
            return;
        }
        if matches!(
            self.state,
            AEGStage::Delay | AEGStage::Attack | AEGStage::Decay
        ) {
            // Protected stages: finish Delay → Attack → Decay → Sustain first,
            // then release (XG short-note behavior). Without this, a very short
            // key press cuts the attack/decay and the note barely sounds.
            self.pending_release = true;
            return;
        }
        self.release_start_level = self.level;
        self.state = AEGStage::Release;
        self.stage_started = self.elapsed_total;
    }

    pub fn kill(&mut self) {
        self.state = AEGStage::Finished;
        self.level = 0.0;
    }

    /// Advance the envelope (called once per block), return current level [0,1]
    pub fn tick(&mut self, elapsed: Duration) -> f32 {
        if !self.enabled {
            return 1.0;
        }
        self.elapsed_total += elapsed;
        let t = self.elapsed_total - self.stage_started;

        match self.state {
            AEGStage::Delay => {
                if self.delay_time.is_zero() || t >= self.delay_time {
                    self.level = 0.0;
                    self.advance(AEGStage::Attack);
                } else {
                    self.level = 0.0;
                }
            }
            AEGStage::Attack => {
                if self.attack_time.is_zero() || t >= self.attack_time {
                    self.level = 1.0;
                    self.advance(AEGStage::Decay);
                } else {
                    self.level = (t.as_secs_f32() * self.inv_attack).min(1.0);
                }
            }
            AEGStage::Decay => {
                if self.decay_time.is_zero() || t >= self.decay_time {
                    self.level = self.sustain_level;
                    self.advance(AEGStage::Sustain);
                } else {
                    let p = t.as_secs_f32() * self.inv_decay;
                    self.level = 1.0 - (1.0 - self.sustain_level) * p;
                }
            }
            AEGStage::Sustain => {
                if self.pending_release {
                    // Attack+Decay completed after a short key press: release now.
                    self.pending_release = false;
                    self.release_start_level = self.level;
                    self.advance(AEGStage::Release);
                } else if self.damper {
                    self.damp_start_level = self.level;
                    self.advance(AEGStage::Damp);
                } else {
                    self.level = self.sustain_level;
                }
            }
            AEGStage::Damp => {
                if self.damp_time.is_zero() || t >= self.damp_time {
                    self.level = 0.0;
                    self.advance(AEGStage::Finished);
                } else {
                    let p = t.as_secs_f32() * self.inv_damp;
                    self.level = (self.damp_start_level * (1.0 - p)).max(0.0);
                }
            }
            AEGStage::Release => {
                if self.release_time.is_zero() || t >= self.release_time {
                    self.level = 0.0;
                    self.advance(AEGStage::Finished);
                } else {
                    let p = t.as_secs_f32() * self.inv_release;
                    self.level = (self.release_start_level * (1.0 - p)).max(0.0);
                }
            }
            AEGStage::Finished => {
                self.level = 0.0;
            }
        }

        self.level
    }

    fn advance(&mut self, next: AEGStage) {
        self.state = next;
        self.stage_started = self.elapsed_total;
    }
}

impl Audio for AEG {
    fn tick(&mut self, elapsed: Duration) -> f32 {
        self.tick(elapsed)
    }
}

/// 08 pp 1A-1C relative time parameter → milliseconds (64=center)
fn param_to_ms(param: u8, base_ms: f32) -> f32 {
    let off = param as f32 - 64.0;
    (base_ms * 1.2f32.powf(off)).clamp(0.1, 20000.0)
}

/// key_on_delay (element[72]) → key-on delay duration (0x500 stage table, frames @ 44100)
fn delay_frames_to_duration(key_on_delay: u8) -> Duration {
    let frames = KEY_ON_DELAY_TABLE[(key_on_delay & 0x7f) as usize];
    Duration::from_secs_f64(frames as f64 / 44100.0)
}

/// 1 / duration_secs (zero-safe: zero duration → 0, the tick's is_zero branch
/// short-circuits before the multiply)
#[inline]
fn inv_secs(d: Duration) -> f32 {
    let s = d.as_secs_f32();
    if s > 0.0 { 1.0 / s } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settled(aeg: &mut AEG, steps: usize, step: Duration) {
        for _ in 0..steps {
            aeg.tick(step);
        }
    }

    #[test]
    fn damper_decays_from_sustain_to_release() {
        let mut aeg = AEG::new();
        aeg.setup(64, 64, 64, 0); // attack 5ms, decay 100ms, release 100ms
        settled(&mut aeg, 200, Duration::from_millis(1));
        assert_eq!(aeg.state, AEGStage::Sustain);
        assert_eq!(aeg.level, aeg.sustain_level);

        aeg.set_damper(true);
        aeg.tick(Duration::from_millis(1));
        assert_eq!(aeg.state, AEGStage::Damp, "damper must enter Damp from Sustain");
        let l0 = aeg.level;
        aeg.tick(Duration::from_millis(100));
        assert!(aeg.level < l0, "Damp stage must decay the level");

        aeg.note_off();
        assert_eq!(aeg.state, AEGStage::Release, "pedal release → Release stage");
        let l1 = aeg.level;
        settled(&mut aeg, 50, Duration::from_millis(2));
        assert_eq!(aeg.state, AEGStage::Finished, "release finishes quickly");
        assert!(aeg.level < l1, "release decays from the current level");
    }

    #[test]
    fn damper_disabled_freezes_at_sustain() {
        let mut aeg = AEG::new();
        aeg.setup(64, 64, 64, 0);
        settled(&mut aeg, 200, Duration::from_millis(1));
        aeg.set_damper(false);
        settled(&mut aeg, 50, Duration::from_millis(10));
        assert_eq!(aeg.state, AEGStage::Sustain);
        assert_eq!(aeg.level, aeg.sustain_level, "no damper → frozen at sustain level");
    }

    #[test]
    fn release_from_decay_uses_current_level() {
        let mut aeg = AEG::new();
        aeg.setup(64, 64, 64, 0);
        settled(&mut aeg, 30, Duration::from_millis(1)); // still inside Decay
        assert_eq!(aeg.state, AEGStage::Decay);
        let l0 = aeg.level;
        assert!(l0 < 1.0 && l0 > 0.0);
        aeg.note_off();
        aeg.tick(Duration::from_millis(1));
        assert!(aeg.level <= l0, "release continues from the current level");
    }

    #[test]
    fn key_on_delay_holds_level_zero_before_attack() {
        let mut aeg = AEG::new();
        aeg.setup(64, 64, 64, 22); // musicbox element: 22 → 112 frames ≈ 2.54 ms
        assert_eq!(aeg.state, AEGStage::Delay, "key-on delay enters Delay stage");
        assert_eq!(aeg.level, 0.0);

        settled(&mut aeg, 1, Duration::from_millis(1));
        assert_eq!(aeg.state, AEGStage::Delay, "still delayed after 1 ms");
        assert_eq!(aeg.level, 0.0, "level must stay 0 during delay");

        settled(&mut aeg, 2, Duration::from_millis(1)); // t=2ms (still delayed), t=3ms (delay over)
        assert_eq!(
            aeg.state,
            AEGStage::Attack,
            "attack starts once the delay elapses"
        );
        assert!(aeg.level >= 0.0 && aeg.level <= 1.0);

        assert_eq!(KEY_ON_DELAY_TABLE[22], 112, "musicbox element 1: 22 → 112 frames");
        assert_eq!(KEY_ON_DELAY_TABLE[21], 104, "musicbox element 2: 21 → 104 frames");
    }

    #[test]
    fn zero_delay_skips_delay_stage() {
        let mut aeg = AEG::new();
        aeg.setup(64, 64, 64, 0);
        aeg.tick(Duration::ZERO);
        assert_eq!(aeg.state, AEGStage::Attack, "key_on_delay=0 must skip Delay");
    }
}

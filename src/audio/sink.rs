#![allow(dead_code)]
/// Audio output target abstraction
///
/// `CpalSink` (real-time) and `VecBufferSink` (tests) accumulate frames and
/// output them in `flush()`; the sink's actual rate drives the render clock.
pub trait AudioSink: std::any::Any {
    /// Push one frame of stereo samples (L, R)
    fn push_frame(&mut self, left: f32, right: f32);
    /// Output accumulated frames. Called once per audio block.
    fn flush(&mut self);
    /// Frame count (for debugging/tests)
    fn frame_count(&self) -> usize;
    /// Actual output sample rate (the render virtual clock must follow this,
    /// or the pitch shifts by requested/actual).
    fn rate(&self) -> u32 {
        48000
    }
    /// Concrete type access (tests/backend configuration)
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn set_debug(&mut self, debug_mode: bool);
}

/// Internal interleaved buffer sink (tests)
#[derive(Debug)]
pub struct VecBufferSink {
    pub buffer: Vec<f32>,
}

impl VecBufferSink {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Take out the buffer (interleaved L,R), and clear it
    pub fn take_buffer(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.buffer)
    }
}

impl Default for VecBufferSink {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioSink for VecBufferSink {
    fn push_frame(&mut self, left: f32, right: f32) {
        self.buffer.push(left);
        self.buffer.push(right);
    }

    fn flush(&mut self) {}

    fn frame_count(&self) -> usize {
        self.buffer.len() / 2
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn set_debug(&mut self, _debug_mode: bool) {}
}

/// Output gain + soft-clip wrapper applied to the final sink
pub struct GainSink {
    inner: Box<dyn AudioSink>,
    gain: f32,
    soft_clip: bool,
}

impl GainSink {
    pub fn new(inner: Box<dyn AudioSink>, gain: f32, soft_clip: bool) -> Self {
        Self {
            inner,
            gain,
            soft_clip,
        }
    }

    pub fn inner_mut(&mut self) -> &mut dyn AudioSink {
        self.inner.as_mut()
    }
}

impl AudioSink for GainSink {
    fn push_frame(&mut self, left: f32, right: f32) {
        let mut l = left * self.gain;
        let mut r = right * self.gain;
        if self.soft_clip {
            l = l.tanh();
            r = r.tanh();
        }
        self.inner.push_frame(l, r);
    }
    fn flush(&mut self) {
        self.inner.flush();
    }
    fn frame_count(&self) -> usize {
        self.inner.frame_count()
    }
    fn rate(&self) -> u32 {
        self.inner.rate()
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn set_debug(&mut self, debug_mode: bool) {
        self.inner.set_debug(debug_mode);
    }
}

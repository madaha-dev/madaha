//! PipeWire output sink
//!
//! The PipeWire mainloop runs in its own thread (all Rc types stay inside it);
//! the render thread pushes interleaved f32 frames into a lock-free ring and
//! the stream process callback reads from it.
use std::sync::{Arc, Mutex};

use pipewire as pw;
use pw::properties::properties;

use crate::config::AudioConfig;

use super::ringbuf::SpscRing;
use crate::audio::sink::AudioSink;

pub struct PipewireSink {
    _rb: Arc<SpscRing>,
    _handle: Option<std::thread::JoinHandle<()>>,
    buffer: Vec<f32>,
    debug_mode: bool,
}

impl PipewireSink {
    pub fn open(cfg: &AudioConfig) -> Result<Self, String> {
        // Decouple the ring from the render block size: a fixed ~1.36s buffer
        // (65536 frames @48k) absorbs scheduler jitter so the consumer never
        // runs dry on small render blocks.
        let rb = Arc::new(SpscRing::new((cfg.buffer_size as usize * 8).max(65536)));
        let rb_cb = rb.clone();
        let rate = cfg.sample_rate;
        let channels = cfg.channels as usize;
        let shared_buf = Arc::new(Mutex::new(vec![0.0f32; cfg.buffer_size as usize * 2 * 8]));
        let shared_cb = shared_buf.clone();

        let handle = std::thread::Builder::new()
            .name("pipewire-mainloop".into())
            .spawn(move || {
                if let Err(e) = Self::run_mainloop(rb_cb, shared_cb, rate, channels) {
                    eprintln!("pipewire mainloop error: {e}");
                }
            })
            .map_err(|e| format!("pipewire thread: {e}"))?;

        Ok(Self {
            _rb: rb,
            _handle: Some(handle),
            buffer: Vec::with_capacity(cfg.buffer_size as usize * 2),
            debug_mode: false
        })
    }

    fn run_mainloop(
        rb: Arc<SpscRing>,
        shared_buf: Arc<Mutex<Vec<f32>>>,
        _rate: u32,
        channels: usize,
    ) -> Result<(), pw::Error> {
        pw::init();
        let mainloop = pw::main_loop::MainLoopRc::new(None)?;
        let context = pw::context::ContextRc::new(&mainloop, None)?;
        let core = context.connect_rc(None)?;

        let props = properties! {
            *pw::keys::MEDIA_TYPE => "Audio",
            *pw::keys::MEDIA_CATEGORY => "Playback",
            *pw::keys::MEDIA_ROLE => "Music",
        };
        let stream = pw::stream::StreamRc::new(core, "madaha", props)?;

        let _listener = stream
            .add_local_listener_with_user_data(())
            .process(move |stream, _: &mut ()| {
                if let Some(mut buffer) = stream.dequeue_buffer() {
                    let datas = buffer.datas_mut();
                    if let Some(data) = datas.first_mut() {
                        if let Some(samples) = data.data() {
                            // Write exactly the frames requested for the current
                            // quantum (`pw_time.size`): the mapped buffer is
                            // sized to the maximum quantum and filling it all
                            // plays back too fast (pitch shifts up by the ratio
                            // quantum_max / quantum).
                            // pw_time.size is the number of SAMPLES requested for the
                            // current quantum (per channel is not included); the
                            // per-callback frame budget is size / channels. Writing
                            // `size` frames plays back at channels× the speed.
                            let quantum = unsafe {
                                let mut ti = std::mem::MaybeUninit::<pw::sys::pw_time>::uninit();
                                if pw::sys::pw_stream_get_time_n(
                                    stream.as_raw_ptr(),
                                    ti.as_mut_ptr(),
                                    std::mem::size_of::<pw::sys::pw_time>(),
                                ) == 0
                                {
                                    ti.assume_init().size as usize / channels
                                } else {
                                    0
                                }
                            };
                            let cap_bytes = samples.len();
                            let frames_cap = cap_bytes / (4 * channels);
                            let want = quantum.min(frames_cap);
                            if want > 0 {
                                let mut tmp = shared_buf.lock().unwrap();
                                if tmp.len() < want * 2 {
                                    tmp.resize(want * 2, 0.0);
                                }
                                let frames = rb.read(&mut tmp[..want * 2]);
                                let n = frames * 2;
                                for i in 0..n {
                                    let offset = i * 4;
                                    if offset + 4 <= samples.len() {
                                        samples[offset..offset + 4]
                                            .copy_from_slice(&tmp[i].to_le_bytes());
                                    }
                                }
                                let chunk = data.chunk_mut();
                                *chunk.offset_mut() = 0;
                                *chunk.size_mut() = (n * 4) as u32;
                                *chunk.stride_mut() = (channels * 4) as i32;
                                if std::fs::metadata("/tmp/pw_fw.txt").is_ok() {
                                    let mut s = String::new();
                                    use std::io::Read;
                                    let _ = std::fs::File::open("/tmp/pw_fw.txt")
                                        .and_then(|mut f| f.read_to_string(&mut s));
                                    if s.lines().count() < 10 {
                                        let _ = std::fs::write(
                                            "/tmp/pw_fw.txt",
                                            format!("{}want={want} got={frames}\n", s),
                                        );
                                    }
                                }
                                if std::fs::metadata("/tmp/pw_dump.bin").is_ok() {
                                    use std::io::Write;
                                    if let Ok(mut f) = std::fs::OpenOptions::new()
                                        .append(true).open("/tmp/pw_dump.bin")
                                    {
                                        let bytes: Vec<u8> = tmp[..n]
                                            .iter()
                                            .flat_map(|v| v.to_le_bytes())
                                            .collect();
                                        let _ = f.write_all(&bytes);
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .register()?;

        // Negotiate f32 output format
        let mut audio_info = pw::spa::param::audio::AudioInfoRaw::new();
        audio_info.set_format(pw::spa::param::audio::AudioFormat::F32LE);
        audio_info.set_rate(_rate);
        audio_info.set_channels(channels as u32);
        let obj = pw::spa::pod::Object {
            type_: pw::spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
            id: pw::spa::param::ParamType::EnumFormat.as_raw(),
            properties: audio_info.into(),
        };
        let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
            std::io::Cursor::new(Vec::new()),
            &pw::spa::pod::Value::Object(obj),
        )
        .expect("serialize format")
        .0
        .into_inner();

        let mut params = [pw::spa::pod::Pod::from_bytes(&values).unwrap()];

        stream.connect(
            pw::spa::utils::Direction::Output,
            None,
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut params,
        )?;

        mainloop.run();
        Ok(())
    }
}

impl AudioSink for PipewireSink {
    fn push_frame(&mut self, left: f32, right: f32) {
        self.buffer.push(left);
        self.buffer.push(right);
    }

    fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        self._rb.write(&self.buffer);
        self.buffer.clear();
    }

    fn frame_count(&self) -> usize {
        self.buffer.len() / 2
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn set_debug(&mut self, debug_mode: bool) {
        self.debug_mode = debug_mode;
    }
}

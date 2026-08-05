//! Jack audio output sink
//!
//! The render thread pushes interleaved f32 frames into a lock-free ring;
//! the Jack process callback (audio thread) reads from it and fills the
//! output ports.
use std::sync::{Arc, Mutex};

use jack::contrib::ClosureProcessHandler;
use jack::{AudioOut, Client, ClientOptions, Port, PortFlags, ProcessScope};

use crate::config::AudioConfig;

use super::ringbuf::SpscRing;
use crate::audio::sink::AudioSink;

pub struct JackSink {
    /// Keep the client alive for the callback's lifetime
    _active: jack::AsyncClient<(), ClosureProcessHandler<(), Box<dyn FnMut(&Client, &ProcessScope) -> jack::Control + Send>>>,
    /// Shared read buffer for the callback (avoids per-block allocation)
    #[allow(dead_code)]
    shared_buf: Arc<Mutex<Vec<f32>>>,
    /// Ring shared with the callback
    _rb: Arc<SpscRing>,
    /// Accumulated frames (render side)
    buffer: Vec<f32>,
}

impl JackSink {
    pub fn open(cfg: &AudioConfig) -> Result<Self, String> {
        let (client, _status) = Client::new(&cfg.jack_client_name, ClientOptions::default())
            .map_err(|e| format!("jack open: {e}"))?;
        let port_l: Port<AudioOut> = client
            .register_port("out_l", AudioOut::default())
            .map_err(|e| format!("jack port_l: {e}"))?;
        let port_r: Port<AudioOut> = client
            .register_port("out_r", AudioOut::default())
            .map_err(|e| format!("jack port_r: {e}"))?;

        let rb = Arc::new(SpscRing::new(cfg.buffer_size as usize * 8));
        let rb_cb = rb.clone();
        let shared_buf = Arc::new(Mutex::new(vec![0.0f32; cfg.buffer_size as usize * 2 * 8]));
        let shared_cb = shared_buf.clone();

        let mut port_l = port_l;
        let mut port_r = port_r;

        // 自动连接物理输出端口 (在激活前)
        let phys = client.ports(
            Some("system"),
            None,
            PortFlags::IS_PHYSICAL | PortFlags::IS_INPUT,
        );
        for name in &phys {
            if let (Ok(l), Ok(r)) = (port_l.name(), port_r.name()) {
                let _ = client.connect_ports_by_name(&l, name);
                let _ = client.connect_ports_by_name(&r, name);
            }
        }

        let process: ClosureProcessHandler<
            (),
            Box<dyn FnMut(&Client, &ProcessScope) -> jack::Control + Send>,
        > = ClosureProcessHandler::new(Box::new(
            move |_client: &Client, ps: &ProcessScope| {
            let n = ps.n_frames() as usize;
            let out_l = port_l.as_mut_slice(ps);
            let out_r = port_r.as_mut_slice(ps);
            let mut buf = shared_cb.lock().unwrap();
            if buf.len() < n * 2 {
                buf.resize(n * 2, 0.0);
            }
            let frames = rb_cb.read(&mut buf[..n * 2]);
            let mut i = 0;
            while i < n {
                out_l[i] = if i < frames { buf[2 * i] } else { 0.0 };
                out_r[i] = if i < frames { buf[2 * i + 1] } else { 0.0 };
                i += 1;
            }
            jack::Control::Continue
            },
        ));

        let active = jack::AsyncClient::new(client, (), process)
            .map_err(|e| format!("jack activate: {e}"))?;

        Ok(Self {
            _active: active,
            shared_buf,
            _rb: rb,
            buffer: Vec::with_capacity(cfg.buffer_size as usize * 2),
        })
    }
}

impl AudioSink for JackSink {
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
}

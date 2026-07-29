pub trait HasSample {
    fn set_wave(&mut self, wave: &Box<[u8]>) -> Self;
}

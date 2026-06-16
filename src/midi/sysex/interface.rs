use super::super::engine::Engine;

pub trait Event {
    fn parse(e: &mut Engine, data: Box<[u8]>);
}

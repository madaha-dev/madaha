use std::time::Duration;

/// 所有发声单元的统一的推进接口（每 block/sample 调用一次）
pub trait Audio {
    fn tick(&mut self, _elapsed: Duration) -> f32 {
        0.0
    }
}

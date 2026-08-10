mod descramble_wave;
mod errors;
mod interface;
mod loader;
mod types;
mod to_f32;

pub mod yxg50;

use descramble_wave::descramble_wave;

pub use errors::LoadError;
pub use interface::SoundModule;
pub use loader::load;
pub use types::LoadedModule;
pub use types::SoundModuleType;

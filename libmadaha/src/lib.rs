mod descramble_wave;
mod errors;
mod interface;
mod types;
mod loader;

pub mod yxg50;

use descramble_wave::descramble_wave;

pub use errors::LoadError;
pub use interface::SoundModule;
pub use types::SoundModuleType;
pub use types::LoadedModule;
pub use loader::load;
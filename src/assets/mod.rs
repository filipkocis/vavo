mod assets;
mod handle;
mod loader;
mod shader;

pub use assets::Assets;
pub use handle::Handle;
pub use loader::{AssetLoader, LoadableAsset};
pub use shader::{Shader, ShaderLoader};

pub trait Asset: Send + Sync + 'static {}

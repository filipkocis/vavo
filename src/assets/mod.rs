mod assets;
mod handle;
mod loader;
pub mod scene;
mod shader;

pub use assets::Assets;
pub use handle::Handle;
pub use loader::{AssetLoader, LoadableAsset};
pub use scene::{Scene, SceneProto};
pub use shader::{Shader, ShaderLoader};

pub trait Asset: Send + Sync + 'static {}

/// Name component, mainly used for scene nodes but can be used as a standalone component to easily
/// identify entities in the ECS
#[derive(vavo_macros::Component, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Name(String);

impl Name {
    /// Create a new Name component
    #[inline]
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self(name.into())
    }

    /// Get the name string
    #[inline]
    pub fn name(&self) -> &str {
        &self.0
    }
}

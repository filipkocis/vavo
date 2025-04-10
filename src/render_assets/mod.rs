mod render_assets;
mod buffer;
mod bind_group;
pub mod pipeline;
mod render_handle;
mod storage;

pub use render_assets::{RenderAssets, IntoRenderAsset, RenderAssetEntry};
pub use buffer::Buffer;
pub use bind_group::BindGroup;
pub use pipeline::{StandardPipeline, Pipeline};
pub use render_handle::RenderHandle;
pub use storage::{Storage, TransformStorage};

pub trait RenderAsset: Send + Sync + 'static {}

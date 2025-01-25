pub mod resources;
pub mod time;

pub use resources::*;
pub use time::*;

/// A type which can be stored as a world resource. Accessed with [`Res`] and [`ResMut`]
///
/// Only one instance of each resource type is allowed per [`World`](super::world::World)
pub trait Resource: Send + Sync + 'static {}

mod macros;
mod proto;

pub use macros::*;
pub use proto::Proto;

use crate::prelude::{Component, EntityId, World};
use std::any::Any;

/// Trait for scene objects which can create a default instance of themselves using
/// [`prototypes`](Proto) with a scene context.
///
/// Has a blanket implementation for any type implementing [`Default`].
///
/// Users should only implement [`proto_build`](SceneProto::proto_build) when implementing this
/// trait, then use the [`proto`](SceneProto::proto) method to create a [Proto] component which can
/// be used in `scene` creation (requires `impl Scene`).
///
/// [Scene] is automatically implemented for any [Component]
/// implementing this trait.
pub trait SceneProto {
    /// Create a default instance of this type for the scene, given the necessary context.
    /// Used as the basis for the [Proto] component.
    fn proto_build(world: &mut World, entity: EntityId) -> Self;

    /// Create a [Proto] component for this type using the `proto_build` method as the builder
    /// function.
    #[inline]
    fn proto() -> Proto<Self>
    where
        Self: Sized,
    {
        Proto::new(|world, entity| Self::proto_build(world, entity))
    }
}

impl<T: Default> SceneProto for T {
    fn proto_build(_: &mut World, _: EntityId) -> Self {
        T::default()
    }
}

/// Trait for scene objects which can build themselves into the ECS world
///
/// Implemented for any [`Component`] which also implements [`SceneProto`].
///
/// Users should implement [`SceneProto`] for their `components`, this trait will then be
/// implemented automatically.
pub trait Scene: Any + Send + Sync + 'static {
    /// Build the scene into the given world under the specified entity.
    /// You can also use
    /// [Commands::insert_scene](crate::system::commands::EntityCommands::insert_scene).
    fn build(&self, world: &mut World, entity: EntityId);
}

impl<T: SceneProto + Component> Scene for T {
    fn build(&self, world: &mut World, entity: EntityId) {
        let component = T::proto_build(world, entity);
        world.insert_component(entity, component, false);
    }
}

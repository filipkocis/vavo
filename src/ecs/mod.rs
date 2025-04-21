pub mod entities;
pub mod resources;
pub mod state;
pub mod world;
pub mod tick;

pub mod ptr;
pub mod store;

pub mod prelude {
    pub use super::entities::{
        components::Component,
        relation::{Children, Parent},
        Entities, EntityId,
    };
    pub use super::resources::{
        FixedTime, Res, ResMut, Resource, Resources, Time, Timer, TimerVariant,
    };
    pub use super::state::{conditions::*, NextState, State, StateTransitionEvent, States};
    pub use super::tick::Tick;
    pub use super::world::World;
}

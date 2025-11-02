pub mod change_detection;
pub mod entities;
pub mod resources;
pub mod state;
pub mod tick;
pub mod world;

pub mod ptr;
pub mod store;

pub mod prelude {
    pub use super::change_detection::ChangeDetection;
    pub use super::entities::{
        Entities, EntityId,
        components::{Component, Mut, Ref},
        relation::{Children, Parent},
    };
    pub use super::resources::{
        FixedTime, FpsCounter, Res, ResMut, Resource, Resources, Time, Timer, TimerVariant,
    };
    pub use super::state::{NextState, State, StateTransitionEvent, States, conditions::*};
    pub use super::tick::Tick;
    pub use super::world::World;
}

pub mod resources;
pub mod state;
pub mod world;
pub mod entities;

pub mod prelude {
    pub use super::resources::{
        Resource,
        Resources, Res, ResMut, 
        Time, FixedTime, Timer, TimerVariant,
    };
    pub use super::state::{
        States, State, NextState,
        StateTransitionEvent, conditions::*,
    };
    pub use super::world::World;
    pub use super::entities::{
        Entities, EntityId, 
        relation::{Children, Parent},
    };
}

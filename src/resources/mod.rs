mod resources;
mod time;

pub use resources::*;
pub use time::{Time, FixedTime, Timer, TimerVariant};

pub trait Resource: Send + Sync + 'static {}

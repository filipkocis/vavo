pub mod systems;
mod event;
pub mod conditions;

pub use event::StateTransitionEvent;
use crate::macros::Resource;

use std::fmt::Debug;

/// Trait representing a state. 
///
/// # Usage
/// ```
/// use vavo::prelude::*;
///
/// #[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq)]
/// enum GameState {
///     #[default]
///     Paused,
///     Playing,
/// }
/// ```
pub trait States: Debug + Clone + Copy + PartialEq + Eq + Send + Sync + 'static {}

/// Current app state
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct State<S: States>(pub S);

/// State to transition to in the next frame
#[derive(Resource, Debug, Clone, Copy)]
pub struct NextState<S: States>(pub Option<S>);

impl<S: States + Default> State<S> {
    pub(crate) fn new() -> Self {
        Self(S::default())
    }
}

impl<S: States> State<S> {
    /// Apply the next state
    pub(crate) fn update(&mut self, value: S) {
        self.0 = value;
    }

    /// Get the state value
    pub fn get(&self) -> S {
        self.0
    }
}

impl<S: States> NextState<S> {
    pub(crate) fn new() -> Self {
        Self(None)
    }

    /// Take out the next state value
    pub(crate) fn take(&mut self) -> Option<S> {
        self.0.take()
    }

    /// Queue a state transition for the next frame
    pub fn set(&mut self, value: S) {
        self.0 = Some(value);
    }

    /// Get the next state value
    pub fn get(&self) -> Option<S> {
        self.0
    }
}

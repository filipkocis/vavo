pub mod systems;
mod event;
pub mod conditions;

pub use event::StateTransitionEvent;

use std::fmt::Debug;

/// Trait representing a state. 
///
/// # Usage
/// ```
/// use engine::prelude::*;
///
/// #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// enum GameState {
///     #[default]
///     Paused,
///     Playing,
/// }
///
/// impl States for GameState {}
/// ```
pub trait States: Debug + Clone + Copy + Default + PartialEq + Eq {}

/// Current app state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct State<S: States>(pub S);

/// State to transition to in the next frame
#[derive(Debug, Clone, Copy)]
pub struct NextState<S: States>(pub Option<S>);

impl<S: States> State<S> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

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

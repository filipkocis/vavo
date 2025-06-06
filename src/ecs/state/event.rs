use super::States;

/// Describes a state transition
pub struct StateTransitionEvent<S: States> {
    pub from: S,
    pub to: S,
}

impl<S: States> StateTransitionEvent<S> {
    pub(super) fn new(from: S, to: S) -> Self {
        Self { from, to }
    }

    /// True if current state is exiting from 'state'
    pub fn exiting(&self, state: S) -> bool {
        self.from == state
    }

    /// True if current state is entering 'state'
    pub fn entering(&self, state: S) -> bool {
        self.to == state
    }
}

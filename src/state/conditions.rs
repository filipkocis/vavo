use crate::prelude::*;

use super::event::StateTransitionEvent;


/// Evaluates to true if the current state is exiting the provided `state`
pub fn on_exit<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.event_reader.read::<StateTransitionEvent<S>>().iter()
            .any(|e| e.exiting(state))
    }
}

/// Evaluates to true if the current state is entering the provided `state`
pub fn on_enter<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.event_reader.read::<StateTransitionEvent<S>>().iter()
            .any(|e| e.entering(state))
    }
}

/// Evaluates to true if any state transition event has occured
pub fn on_transition<S: States + 'static>() -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.event_reader.read::<StateTransitionEvent<S>>().len() > 0
    }
}

/// Evaluates to true if the current state is `state`
pub fn in_state<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.resources.get::<State<S>>().map_or(false, |s| s.get() == state)
    }
}

/// Evaluates to true if the current state is not `state`
pub fn not_in_state<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.resources.get::<State<S>>().map_or(false, |s| s.get() != state)
    }
}

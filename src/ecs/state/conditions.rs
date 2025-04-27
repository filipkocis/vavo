use crate::prelude::*;

use super::event::StateTransitionEvent;

/// Evaluates to true if the current state is exiting the provided `state`
pub fn on_exit<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.event_reader
            .read::<StateTransitionEvent<S>>()
            .iter()
            .any(|e| e.exiting(state))
    }
}

/// Evaluates to true if the current state is entering the provided `state`
pub fn on_enter<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.event_reader
            .read::<StateTransitionEvent<S>>()
            .iter()
            .any(|e| e.entering(state))
    }
}

/// Evaluates to true if any state transition event has occured
pub fn on_transition<S: States + 'static>() -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| ctx.event_reader.read::<StateTransitionEvent<S>>().len() > 0
}

/// Evaluates to true if the current state is `state`
pub fn in_state<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.resources
            .get::<State<S>>()
            .map_or(false, |s| s.get() == state)
    }
}

/// Evaluates to true if the current state is not `state`
pub fn not_in_state<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.resources
            .get::<State<S>>()
            .map_or(true, |s| s.get() != state)
    }
}

/// Negates the result of the provided condition
pub fn not<T: 'static, F: 'static>(
    condition: impl IntoSystemCondition<T, F>,
) -> impl IntoSystemCondition<T, F> {
    let condition = condition.system_condition();
    move |ctx: &mut SystemsContext, query: Query<T, F>| !condition(ctx, query)
}

/// Evaluates to true if any events of type `E` have been sent
pub fn on_event<E: 'static>(ctx: &mut SystemsContext, _: Query<(), ()>) -> bool {
    ctx.event_reader.has_any::<E>()
}

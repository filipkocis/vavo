use std::time::Duration;

use crate::prelude::*;

/// Creates a [Condition](IntoSystemCondition) which evaluates to true if the current state is
/// exiting the provided `state`
pub fn on_exit<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.event_reader
            .read::<StateTransitionEvent<S>>()
            .iter()
            .any(|e| e.exiting(state))
    }
}

/// Creates a  [Condition](IntoSystemCondition) which evaluates to true if the current state is
/// entering the provided `state`
pub fn on_enter<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.event_reader
            .read::<StateTransitionEvent<S>>()
            .iter()
            .any(|e| e.entering(state))
    }
}

/// [Condition](IntoSystemCondition) which evaluates to true if any state transition event has occured
pub fn on_transition<S: States + 'static>(ctx: &mut SystemsContext, _: Query<(), ()>) -> bool {
    ctx.event_reader.has_any::<StateTransitionEvent<S>>()
}

/// Creates a [Condition](IntoSystemCondition) which evaluates to true if the current state is `state`
pub fn in_state<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.resources
            .get::<State<S>>()
            .map_or(false, |s| s.get() == state)
    }
}

/// Creates a [Condition](IntoSystemCondition) which evaluates to true if the current state is not `state`
pub fn not_in_state<S: States + 'static>(state: S) -> impl IntoSystemCondition<(), ()> {
    move |ctx: &mut SystemsContext, _| {
        ctx.resources
            .get::<State<S>>()
            .map_or(true, |s| s.get() != state)
    }
}

/// Creates a [Condition](IntoSystemCondition) which negates the result of the provided condition
pub fn not<T: 'static, F: 'static>(
    condition: impl IntoSystemCondition<T, F>,
) -> impl IntoSystemCondition<T, F> {
    let mut condition = condition.system_condition();
    move |ctx: &mut SystemsContext, query: Query<T, F>| !condition(ctx, query)
}

/// [Condition](IntoSystemCondition) which evaluates to true if any events of type `E` have been sent
pub fn on_event<E: 'static>(ctx: &mut SystemsContext, _: Query<(), ()>) -> bool {
    ctx.event_reader.has_any::<E>()
}

/// [Condition](IntoSystemCondition) which evaluates to true if resource `R` has changed, or false
/// if it doesn't exist
pub fn resource_changed<R: Resource>(ctx: &mut SystemsContext, _: Query<(), ()>) -> bool {
    ctx.resources.get::<R>().map_or(false, |r| r.has_changed())
}

/// [Condition](IntoSystemCondition) which evaluates to true if a resource `R` has been inserted,
/// or false if it doesn't exist
pub fn resource_added<R: Resource>(ctx: &mut SystemsContext, _: Query<(), ()>) -> bool {
    ctx.resources.get::<R>().map_or(false, |r| r.was_added())
}

/// [Condition](IntoSystemCondition) which evaluates to true if resource `R` exists
pub fn resource_exists<R: Resource>(ctx: &mut SystemsContext, _: Query<(), ()>) -> bool {
    ctx.resources.get::<R>().is_some()
}

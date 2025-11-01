use std::time::Duration;

use crate::{
    event::EventReader,
    prelude::*,
    system::{IntoSystemCondition, SystemParam},
};

/// Creates a [Condition](IntoSystemCondition) which evaluates to true if the current state is
/// exiting the provided `state`
pub fn on_exit<S: States + 'static>(
    state: S,
) -> impl IntoSystemCondition<EventReader<StateTransitionEvent<S>>> {
    let closure = move |transition_events: EventReader<StateTransitionEvent<S>>| {
        transition_events.read().iter().any(|e| e.exiting(state))
    };
    closure.build()
}

/// Creates a  [Condition](IntoSystemCondition) which evaluates to true if the current state is
/// entering the provided `state`
pub fn on_enter<S: States + 'static>(
    state: S,
) -> impl IntoSystemCondition<EventReader<StateTransitionEvent<S>>> {
    let closure = move |trasition_events: EventReader<StateTransitionEvent<S>>| {
        trasition_events.read().iter().any(|e| e.entering(state))
    };
    closure.build()
}

/// [Condition](IntoSystemCondition) which evaluates to true if any state transition event has occured
pub fn on_transition<S: States + 'static>(
    event_reader: EventReader<StateTransitionEvent<S>>,
) -> bool {
    event_reader.has_any()
}

/// Creates a [Condition](IntoSystemCondition) which evaluates to true if the current state is `state`
pub fn in_state<S: States + 'static>(state: S) -> impl IntoSystemCondition<Option<Res<State<S>>>> {
    let closure = move |res: Option<Res<State<S>>>| res.is_some_and(|s| s.get() == state);
    closure.build()
}

/// Creates a [Condition](IntoSystemCondition) which evaluates to true if the current state is not `state`
pub fn not_in_state<S: States + 'static>(
    state: S,
) -> impl IntoSystemCondition<Option<Res<State<S>>>> {
    let closure = move |res: Option<Res<State<S>>>| res.is_none_or(|s| s.get() != state);
    closure.build()
}

/// Creates a [Condition](IntoSystemCondition) which negates the result of the provided condition
/// TODO: Because of the current implementation, it requires 'world' param, thus making the system
/// where this is used not parallel. This should be changed in the future.
pub fn not<Params: SystemParam>(
    condition: impl IntoSystemCondition<Params>,
) -> impl IntoSystemCondition<&'static mut World> {
    let mut condition = condition.build();
    let closure = move |world: &mut World| !condition.run(world);
    closure.build()
}

/// [Condition](IntoSystemCondition) which evaluates to true if any events of type `E` have been sent
pub fn on_event<E: Event>(event_reader: EventReader<E>) -> bool {
    event_reader.has_any()
}

/// [Condition](IntoSystemCondition) which evaluates to true if resource `R` has changed, or false
/// if it doesn't exist
pub fn resource_changed<R: Resource>(resource: Option<Res<R>>) -> bool {
    resource.is_some_and(|r| r.has_changed())
}

/// [Condition](IntoSystemCondition) which evaluates to true if a resource `R` has been inserted,
/// or false if it doesn't exist
pub fn resource_added<R: Resource>(resource: Option<Res<R>>) -> bool {
    resource.is_some_and(|r| r.was_added())
}

/// [Condition](IntoSystemCondition) which evaluates to true if resource `R` exists
pub fn resource_exists<R: Resource>(resource: Option<Res<R>>) -> bool {
    resource.is_some()
}

use crate::{event::EventWriter, prelude::*};

use super::event::StateTransitionEvent;

/// State transitioning system, one per state type. Used in the FrameEnd system stage.
pub fn apply_state_transition<S: States>(
    current_state: Option<ResMut<State<S>>>,
    next_state: Option<ResMut<NextState<S>>>,
    mut transition_events: EventWriter<StateTransitionEvent<S>>,
) {
    // resource option
    let Some(mut next_state) = next_state else {
        return;
    };
    let next_state = next_state.take();
    // next state option
    let Some(next_state) = next_state else { return };

    // resource option
    let Some(mut current_state) = current_state else {
        return;
    };

    if current_state.get() == next_state {
        return;
    }

    // queue event and update state
    transition_events.write(StateTransitionEvent::new(current_state.get(), next_state));
    current_state.update(next_state);
}

/// State transitioning system, one per state type. Used in the FrameEnd system stage.
pub fn register_state_events<S: States>(app: &mut App) {
    app.register_event::<StateTransitionEvent<S>>();
}

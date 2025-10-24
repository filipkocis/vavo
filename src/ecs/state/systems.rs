use crate::prelude::*;

use super::event::StateTransitionEvent;

/// State transitioning system, one per state type. Used in the FrameEnd system stage.
pub fn apply_state_transition<S: States + 'static>(ctx: &mut SystemsContext, _: Query<()>) {
    let current_state = ctx.resources.try_get_mut::<State<S>>();     
    let next_state = ctx.resources.try_get_mut::<NextState<S>>();

    // resource option
    let Some(mut next_state) = next_state else { 
        return
    };
    let next_state = next_state.take();
    // next state option
    let Some(next_state) = next_state else {
        return
    };

    // resource option
    let Some(mut current_state) = current_state else { 
        return
    };

    if current_state.get() == next_state {
        return
    }

    // queue event and update state
    ctx.event_writer.send(StateTransitionEvent::new(current_state.get(), next_state));
    current_state.update(next_state);
}

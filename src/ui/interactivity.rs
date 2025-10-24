use std::collections::HashMap;

use winit::event::MouseButton;

use crate::{prelude::*, ui::prelude::*};

/// Marks an UI entity as interactive, enabling mouse events via `Interaction`
#[derive(Component, Debug, Clone, Copy)]
pub struct Button;

/// Enables mouse event tracking for an UI entity, automatically added with `Button`
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    /// Mouse is hovering over the node's bounding box
    Hover,
    /// Left mouse button is held down
    Press,
    #[default]
    None,
}

/// System to update UI interactions, runs in the First stage. So old computed values are used
pub fn ui_interaction_update(
    ctx: &mut SystemsContext, 
    mut query: Query<(EntityId, &Node, &ComputedNode, &GlobalTransform, &Interaction)>
) {
    let nodes = query.iter_mut();
    if nodes.is_empty() {
        // nothing to interact with
        return;
    }

    // clear old interactions, mark non-Nones as None
    let mut interactions: HashMap<EntityId, Interaction> = nodes.iter().filter_map(|e| {
        let interaction = *e.4;
        if interaction == Interaction::None {
            None
        } else {
            Some((e.0, Interaction::None))
        }
    }).collect();

    // new interactions
    let (new_interactions, keep) = match get_interactions(ctx, &nodes) {
        Some(interactions) => interactions,
        None => return,
    };

    interactions.extend(new_interactions);
    interactions.retain(|id, _| !keep.contains(id)); // remove keepers, so they are not updated

    // update interactions
    let mut interaction_query = query.cast::<&mut Interaction, ()>();
    for (id, new_interaction) in interactions {
        let interaction = interaction_query.get(id).expect("Interaction component not found");
        *interaction = new_interaction;
    }
}

/// Get nodes with new interactions
fn get_interactions(
    ctx: &mut SystemsContext,
    nodes: &[(EntityId, &Node, &ComputedNode, &GlobalTransform, &Interaction)],
) -> Option<(
        Vec<(EntityId, Interaction)>, // new
        Vec<EntityId>, // keep
    )> {
    let mouse_inputs = ctx.resources.get::<Input<MouseButton>>();

    let input_events = ctx.event_reader.read::<MouseInput>();
    let move_events = ctx.event_reader.read::<CursorMoved>();

    if input_events.is_empty() && move_events.is_empty() {
        // no events to process
        return None;
    }

    let is_pressed = mouse_inputs.pressed(MouseButton::Left);
    let just_pressed = mouse_inputs.just_pressed(MouseButton::Left);

    let cursor_position = match ctx.renderer.cursor_position() {
        Some(position) => position,
        // cursor is outside of the window, so reset interactions
        None => return Some((vec![], vec![])),
    };

    let mut interactions = Vec::new();
    let mut keep = Vec::new();

    // find intersections
    for (id, node, computed, global_transform, interaction) in nodes {
        // check visibility
        if node.display == Display::None {
            continue;
        }

        // calculate padding bounding box
        let translation = global_transform.translation();
        let left = translation.x + computed.margin.left + computed.border.left;
        let top = translation.y + computed.margin.top + computed.border.top;
        let right = left + computed.width.content + computed.padding.horizontal();
        let bottom = top + computed.height.content + computed.padding.vertical();
        let padding_box = Rect::new_min_max(left, top, right, bottom);
        let hovering =  padding_box.contains(cursor_position);

        let state = match (**interaction, hovering, is_pressed, just_pressed) {
            // hovering
            (_, true, _, true) => Interaction::Press,
            (Interaction::Press, true, true, _) => Interaction::Press,
            (_, true, false, false) => Interaction::Hover,
            (_, true, _, _) => Interaction::Hover,
            
            // not hovering
            (Interaction::Press, false, true, false) => Interaction::Press,

            // we dont need to update to None state, its automatic
            (_, false, _, _) => continue,
        };

        // only add new non-none interactions
        if state != **interaction {
            interactions.push((*id, state));
        } else {
            keep.push(*id)
        }
    }

    Some((interactions, keep))
}

use glam::Vec3;
use glyphon::FontSystem;

use crate::{prelude::*, render_assets::RenderAssets, ui::prelude::*, ui::text::TextBuffer};

use super::build_temp::{nodes_to_temp_graph, TempNode};

/// Post update system to compute ui nodes and update their transforms
pub fn compute_nodes_and_transforms(ctx: &mut SystemsContext, mut q: Query<()>) {
    let mut root_temp_nodes = nodes_to_temp_graph(ctx, &mut q);

    if root_temp_nodes.is_empty() {
        return;
    }

    let mut font_system = ctx.resources.get_mut::<FontSystem>().expect("FontSystem resource not found");
    let mut text_buffers = ctx.resources.get_mut::<RenderAssets<TextBuffer>>().expect("TextBuffer render assets not found");
    resolve_z_index(ctx, &mut text_buffers, &mut root_temp_nodes, &mut 0);

    let screen_width = ctx.renderer.size().width as f32;
    let screen_height = ctx.renderer.size().height as f32;
    
    for node in &mut root_temp_nodes {
        
    }
}

/// Sorts nodes by z_index and then computes the z_index with depth first search.
/// Starts with layer 0, increments by 1 for each node.
///
/// # Important
/// When setting z_index on text, it will recreate the text buffer render asset with new metadata.
fn resolve_z_index(
    ctx: &mut SystemsContext, 
    text_buffers: &mut RenderAssets<TextBuffer>, 
    nodes: &mut Vec<TempNode>, 
    layer: &mut usize
) {
    nodes.sort_by(|a, b| a.node.z_index.cmp(&b.node.z_index)); 

    for node in nodes {
        if let Some(ref mut text) = node.text {
            text.attrs(text.attrs.metadata(*layer + 1)); // +1 to fix LessEqual depthmap issues 
            
            // simply remove the render asset, to recreate it with the new metadata, since buffer
            // does not have a `set_attrs` method, bufferlines do, but it gets reset
            text_buffers.remove_by_entity(&node.id, &**text);
            let text_rae = text_buffers.get_by_entity(&node.id, &**text, ctx);
            node.text_rae = Some(text_rae);
        }

        node.computed.z_index = *layer as i32;
        *layer += 1;

        resolve_z_index(ctx, text_buffers, &mut node.children, layer);
    }
}

impl TempNode<'_> {

}

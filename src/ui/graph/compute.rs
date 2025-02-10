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
        node.measure_intrinsic_size(ctx);
        node.compute_percent_size(screen_width, screen_height);
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
    /// Measures the intrinsic size of the node, and sets the computed content size
    /// Traversal: BOTTOM UP
    fn measure_intrinsic_size(&mut self, ctx: &mut SystemsContext) {
        let mut total_width = 0.0;
        let mut total_height = 0.0;
        let mut total_base_width = 0.0;

        let mut max_width = 0.0f32;
        let mut max_height = 0.0f32;
        let mut max_base_width = 0.0f32;

        for child in &mut self.children {
            child.measure_intrinsic_size(ctx);

            total_width += child.computed.width.total;
            total_height += child.computed.height.total;
            total_base_width += child.computed.base_width;

            max_width = max_width.max(child.computed.width.total);
            max_height = max_height.max(child.computed.height.total);
            max_base_width = max_base_width.max(child.computed.base_width);
        }

        let (base_width, width) = if self.node.width == Val::Auto {
            let text_width = self.text_rae.as_ref()
                .map(|rae| rae.width())
                .unwrap_or_default();

            match (
                self.node.display,
                self.node.flex_direction.is_row(),
            ) {
                (Display::Flex, true) => (total_base_width, total_width + text_width),
                (Display::Block, _) |
                (Display::Flex, false) => (max_base_width, max_width.max(text_width)),
                (Display::None, _) => (0.0, 0.0),
                (Display::Grid, _) => unimplemented!("Grid auto size"),
            }
        } else {
            let val = self.node.width.compute_val(0.0, ctx);
            (val, val)
        };

        let height = if self.node.height == Val::Auto {
            let text_height = self.text_rae.as_ref()
                .map(|rae| rae.height())
                .unwrap_or_default();

            match (
                self.node.display,
                self.node.flex_direction.is_row(),
            ) {
                (Display::Flex, true) => max_height.max(text_height),
                (Display::Block, _) |
                (Display::Flex, false) => total_height + text_height,
                (Display::None, _) => 0.0,
                (Display::Grid, _) => unimplemented!("Grid auto size"),
            }
        } else {
            self.node.height.compute_val(0.0, ctx)
        };

        self.computed.width.set(width); 
        self.computed.height.set(height);
        self.computed.base_width = base_width;
    }

    /// Computes percent size based on parent size
    /// Traversal: TOP DOWN
    fn compute_percent_size(&mut self, parent_width: f32, parent_height: f32) {
        if let Val::Percent(val) = self.node.width {
            self.computed.width.set(parent_width * val / 100.0);
        } 

        if let Val::Percent(val) = self.node.height {
            self.computed.height.set(parent_height * val / 100.0);
        }

        for child in &mut self.children {
            child.compute_percent_size(self.computed.width.content, self.computed.height.content);
        }
    }
}

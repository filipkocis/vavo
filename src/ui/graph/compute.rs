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
        node.compute_auto_size();
        node.compute_percent_size(screen_width, screen_height); // recompute after auto size
        
        node.compute_translation();
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

    /// Computes auto size based on children
    /// Traversal: BOTTOM UP
    fn compute_auto_size(&mut self) {
        if self.children.is_empty() {
            return;
        }

        let mut total_width = 0.0;
        let mut total_height = 0.0;

        let mut max_width = 0.0f32;
        let mut max_height = 0.0f32;

        for child in &mut self.children {
            child.compute_auto_size();

            total_width += child.computed.width.total;
            total_height += child.computed.height.total;

            max_width = max_width.max(child.computed.width.total);
            max_height = max_height.max(child.computed.height.total);
        }

        if self.node.width == Val::Auto {
            match (
                self.node.display,
                self.node.flex_direction.is_row(),
            ) {
                (Display::Flex, true) => self.computed.width.set(total_width),
                (Display::Block, _) |
                (Display::Flex, false) => self.computed.width.set(max_width),
                (Display::None, _) => self.computed.width.set(0.0),
                (Display::Grid, _) => unimplemented!("Grid auto size"),
            }
        }

        if self.node.height == Val::Auto {
            match (
                self.node.display,
                self.node.flex_direction.is_row(),
            ) {
                (Display::Flex, true) => self.computed.height.set(max_height),
                (Display::Block, _) |
                (Display::Flex, false) => self.computed.height.set(total_height),
                (Display::None, _) => self.computed.height.set(0.0),
                (Display::Grid, _) => unimplemented!("Grid auto size"),
            }
        }
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

    /// Computes min / max values for one node
    fn compute_min_max(&mut self, ctx: &mut SystemsContext, parent: Option<*mut TempNode>) {
        let (parent_content_width, parent_content_height) = if let Some(parent) = parent {
            let parent = unsafe { &mut *parent };
            (parent.computed.width.content, parent.computed.height.content) 
        } else {
            let size = ctx.renderer.size();
            (size.width as f32, size.height as f32)
        };

        self.computed.min_width = self.node.min_width.compute_val(parent_content_width, ctx);
        self.computed.max_width = self.node.max_width.compute_val(parent_content_width, ctx);
        self.computed.min_height = self.node.min_height.compute_val(parent_content_height, ctx);
        self.computed.max_height = self.node.max_height.compute_val(parent_content_height, ctx);

        if self.node.max_width == Val::Auto {
            self.computed.max_width = f32::INFINITY;
        }

        if self.node.max_height == Val::Auto {
            self.computed.max_height = f32::INFINITY;
        }

        self.computed.width.set(self.computed.width.content.min(self.computed.max_width).max(self.computed.min_width));
        self.computed.height.set(self.computed.height.content.min(self.computed.max_height).max(self.computed.min_height));
    }

    /// Computes the translations (screen space position)
    /// Traversal: BOTTOM UP
    fn compute_translation(&mut self) {
        // apply box offset
        let self_offset_x = self.computed.border.left
                    + self.computed.padding.left;
        let self_offset_y = self.computed.border.top
                    + self.computed.padding.top;

        self.transform.translation = Vec3::new(
            self.computed.margin.left, 
            self.computed.margin.top, 
            self.computed.z_index as f32
        );

        match self.node.display {
            Display::None => {},
            Display::Grid => unimplemented!("Grid translation"),
            Display::Block => {
                let offset_x = self_offset_x;
                let mut offset_y = self_offset_y;

                for child in &mut self.children {
                    child.compute_translation();
                    child.transform.translation.x += offset_x;
                    child.transform.translation.y += offset_y;

                    offset_y += child.computed.height.total;
                }
            },
            Display::Flex => {
                let justify_content_offsets = self.justify_content_offsets();
                let align_items_offsets = self.align_items_offsets();
                let is_reverse = self.node.flex_direction.is_reverse();

                let mut offset_x = self_offset_x;
                let mut offset_y = self_offset_y;

                for mut i in 0..self.children.len() {
                    if is_reverse {
                        i = self.children.len() - 1 - i;
                    } 
                    let child = &mut self.children[i];
                
                    let justify_content_offset = justify_content_offsets[i];
                    let align_items_offset = align_items_offsets[i];

                    child.compute_translation();
                    child.transform.translation.x += offset_x;
                    child.transform.translation.y += offset_y;

                    child.transform.translation += justify_content_offset;
                    child.transform.translation += align_items_offset;

                    if self.node.flex_direction.is_row() {
                        offset_x += child.computed.width.total
                            + self.computed.column_gap;
                    } else {
                        offset_y += child.computed.height.total
                            + self.computed.row_gap;
                    }
                }
            }
        } 
    }

    /// Returns the offsets for `align-items` for each child node.
    /// `result.len() == self.children.len()`
    ///
    /// Only the main cross-axis field is used in a flex container, otherwise fields are 0.0.
    fn align_items_offsets(&self) -> Vec<Vec3> {
        if self.node.display != Display::Flex {
            return self.offsets_from(Vec3::ZERO);
        }

        match self.node.flex_direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                self.children.iter().map(|child| {
                    let diff = self.computed.height.content - child.computed.height.total;
                    let offset = match self.node.align_items {
                        AlignItems::FlexStart => 0.0,
                        AlignItems::FlexEnd => diff,
                        AlignItems::Center => diff / 2.0,
                        AlignItems::Stretch => 0.0,
                    };
                    Vec3::new(0.0, offset, 0.0)
                }).collect()
            },
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                self.children.iter().map(|child| {
                    let diff = self.computed.width.content - child.computed.width.total;
                    let offset = match self.node.align_items {
                        AlignItems::FlexStart => 0.0,
                        AlignItems::FlexEnd => diff,
                        AlignItems::Center => diff / 2.0,
                        AlignItems::Stretch => 0.0,
                    };
                    Vec3::new(offset, 0.0, 0.0)
                }).collect()
            }
        }
    }

    /// Internal calculation for `justify_content_offsets` method 
    fn justify_content_offsets_internal(&self, is_row: bool, gaps_num: f32) -> Vec<Vec3> {
        let new_vec3 = |val: f32| {
            if is_row {
                Vec3::new(val, 0.0, 0.0)
            } else {
                Vec3::new(0.0, val, 0.0)
            }
        };

        let computed_gap = if is_row {
            self.computed.column_gap
        } else {
            self.computed.row_gap
        };

        let content_size = self.children.iter().fold(gaps_num * computed_gap, |acc, child| 
            acc + if is_row {
                child.computed.width.total
            } else {
                child.computed.height.total
            }
        );

        let offset = if is_row {
            self.computed.width.content
        } else {
            self.computed.height.content
        } - content_size;

        match self.node.justify_content {
            JustifyContent::FlexStart => return self.offsets_from(Vec3::ZERO),
            JustifyContent::FlexEnd => return self.offsets_from(new_vec3(offset)),
            JustifyContent::Center => return self.offsets_from(new_vec3(offset / 2.0)), 
            JustifyContent::SpaceBetween => {
                let offset = offset.max(0.0);
                let between_gap = offset / gaps_num;
                self.children.iter().enumerate().map(|(i, _)| {
                    let gap = if i == 0 { 0.0 } else { between_gap * i as f32 };
                    new_vec3(gap)
                }).collect()
            },
            JustifyContent::SpaceAround => {
                let offset = offset.max(0.0);
                let around_gap = offset / (gaps_num + 1.0);  
                self.children.iter().enumerate().map(|(i, _)| {
                    let gap = (around_gap * i as f32) + around_gap / 2.0;
                    new_vec3(gap)
                }).collect()
            }
            JustifyContent::SpaceEvenly => {
                let offset = offset.max(0.0);
                let even_gap = offset / (gaps_num + 2.0);  
                self.children.iter().enumerate().map(|(i, _)| {
                    let gap = even_gap * (i + 1) as f32;
                    new_vec3(gap)
                }).collect()
            }
        }
    }


    /// Returns the offsets for `justify-content` for each child node.
    /// `result.len() == self.children.len()`
    ///
    /// Only the main axis field is used in a flex container, otherwise fields are 0.0.
    fn justify_content_offsets(&self) -> Vec<Vec3> {
        if self.node.display != Display::Flex {
            return self.offsets_from(Vec3::ZERO); 
        }

        let gaps_num = (self.children.len() as isize - 1).max(0) as f32;

        if self.node.flex_direction.is_row() {
            self.justify_content_offsets_internal(true, gaps_num)
        } else {
            self.justify_content_offsets_internal(false, gaps_num)
        }
    }

    /// Utility function used to create offsets of the same value for `align_items` and
    /// `justify_content`
    fn offsets_from(&self, v: Vec3) -> Vec<Vec3> {
        (0..self.children.len()).map(|_| v).collect::<Vec<_>>()
    }
}

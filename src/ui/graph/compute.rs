use glam::Vec3;
use glyphon::FontSystem;
use winit::dpi::PhysicalSize;

use crate::{
    event::event_handler::EventReader,
    prelude::*,
    render_assets::RenderAssets,
    ui::{prelude::*, text::TextBuffer},
};

use super::build_temp::{TempNode, nodes_to_temp_graph};

/// Post update system to compute ui nodes and update their transforms
pub fn compute_nodes_and_transforms(
    mut q: Query<()>,

    world: &mut World,
    event_reader: EventReader,
    mut font_system: ResMut<FontSystem>,
    mut text_buffers: ResMut<RenderAssets<TextBuffer>>,
) {
    let mut root_temp_nodes = nodes_to_temp_graph(event_reader, &mut q);

    if root_temp_nodes.is_empty() {
        return;
    }

    resolve_z_index(world, &mut text_buffers, &mut root_temp_nodes, &mut 0);

    let window_size = ctx.renderer.size();
    let screen_width = window_size.width as f32;
    let screen_height = window_size.height as f32;

    for node in &mut root_temp_nodes {
        node.measure_intrinsic_size(window_size);
        node.compute_percent_size(screen_width, screen_height);
        node.compute_auto_size();
        node.compute_percent_size(screen_width, screen_height); // recompute after auto size

        node.apply_constraints(window_size, None);
        node.compute_gaps(window_size);
        node.resolve_text_wrap(&mut font_system);
        node.fit_auto_size();

        node.resolve_flex();
        node.recalculate_percent_size();
        // TODO: wrap text after percent width change and readjust auto heights

        node.compute_translation();
    }
}

/// Sorts nodes by z_index and then computes the z_index with depth first search.
/// Starts with layer 0, increments by 1 for each node.
///
/// # Important
/// When setting z_index on text, it will recreate the text buffer render asset with new metadata.
fn resolve_z_index(
    world: &mut World,
    text_buffers: &mut RenderAssets<TextBuffer>,
    nodes: &mut Vec<TempNode>,
    layer: &mut usize,
) {
    nodes.sort_by(|a, b| a.node.z_index.cmp(&b.node.z_index));

    for node in nodes {
        if let Some(ref mut text) = node.text {
            text.attrs.metadata = *layer + 1; // +1 to fix LessEqual depthmap issues 
            

            // simply remove the render asset, to recreate it with the new metadata, since buffer
            // does not have a `set_attrs` method, bufferlines do, but it gets reset
            text_buffers.remove_by_entity(node.id, &**text);
            let text_rae = text_buffers.get_by_entity(node.id, &**text, world);
            node.text_rae = Some(text_rae);
        }

        node.computed.z_index = *layer as i32;
        *layer += 1;

        resolve_z_index(world, text_buffers, &mut node.children, layer);
    }
}

impl TempNode<'_> {
    /// Measures the intrinsic size of the node, and sets the computed content size
    /// Traversal: BOTTOM UP
    fn measure_intrinsic_size(&mut self, window_size: PhysicalSize<u32>) {
        let mut total_width = 0.0;
        let mut total_height = 0.0;
        let mut total_base_width = 0.0;

        let mut max_width = 0.0f32;
        let mut max_height = 0.0f32;
        let mut max_base_width = 0.0f32;

        for child in &mut self.children {
            child.measure_intrinsic_size(window_size);

            total_width += child.computed.width.total;
            total_height += child.computed.height.total;
            total_base_width += child.computed.base_width;

            max_width = max_width.max(child.computed.width.total);
            max_height = max_height.max(child.computed.height.total);
            max_base_width = max_base_width.max(child.computed.base_width);
        }

        let (base_width, width) = if self.node.width == Val::Auto {
            let text_width = self
                .text_rae
                .as_ref()
                .map(|rae| rae.width())
                .unwrap_or_default();

            match (self.node.display, self.node.flex_direction.is_row()) {
                (Display::Flex, true) => (total_base_width, total_width + text_width),
                (Display::Block, _) | (Display::Flex, false) => {
                    (max_base_width, max_width.max(text_width))
                }
                (Display::None, _) => (0.0, 0.0),
                (Display::Grid, _) => unimplemented!("Grid auto size"),
            }
        } else {
            let val = self.node.width.compute_val(0.0, window_size);
            (val, val)
        };

        let height = if self.node.height == Val::Auto {
            let text_height = self
                .text_rae
                .as_ref()
                .map(|rae| rae.height())
                .unwrap_or_default();

            match (self.node.display, self.node.flex_direction.is_row()) {
                (Display::Flex, true) => max_height.max(text_height),
                (Display::Block, _) | (Display::Flex, false) => total_height + text_height,
                (Display::None, _) => 0.0,
                (Display::Grid, _) => unimplemented!("Grid auto size"),
            }
        } else {
            self.node.height.compute_val(0.0, window_size)
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
            match (self.node.display, self.node.flex_direction.is_row()) {
                (Display::Flex, true) => self.computed.width.set(total_width),
                (Display::Block, _) | (Display::Flex, false) => self.computed.width.set(max_width),
                (Display::None, _) => self.computed.width.set(0.0),
                (Display::Grid, _) => unimplemented!("Grid auto size"),
            }
        }

        if self.node.height == Val::Auto {
            match (self.node.display, self.node.flex_direction.is_row()) {
                (Display::Flex, true) => self.computed.height.set(max_height),
                (Display::Block, _) | (Display::Flex, false) => {
                    self.computed.height.set(total_height)
                }
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
    fn compute_min_max(&mut self, window_size: PhysicalSize<u32>, parent: Option<*mut TempNode>) {
        let (parent_content_width, parent_content_height) = if let Some(parent) = parent {
            let parent = unsafe { &mut *parent };
            (
                parent.computed.width.content,
                parent.computed.height.content,
            )
        } else {
            (window_size.width as f32, window_size.height as f32)
        };

        let ws = window_size;
        let pcw = parent_content_width;
        let pch = parent_content_height;
        self.computed.min_width = self.node.min_width.compute_val(pcw, ws);
        self.computed.max_width = self.node.max_width.compute_val(pcw, ws);
        self.computed.min_height = self.node.min_height.compute_val(pch, ws);
        self.computed.max_height = self.node.max_height.compute_val(pch, ws);

        if self.node.max_width == Val::Auto {
            self.computed.max_width = f32::INFINITY;
        }

        if self.node.max_height == Val::Auto {
            self.computed.max_height = f32::INFINITY;
        }

        self.computed.width.set(
            self.computed
                .width
                .content
                .min(self.computed.max_width)
                .max(self.computed.min_width),
        );
        self.computed.height.set(
            self.computed
                .height
                .content
                .min(self.computed.max_height)
                .max(self.computed.min_height),
        );
    }

    /// Computes the box sizing for one node
    fn compute_box_sizing(
        &mut self,
        window_size: PhysicalSize<u32>,
        parent: Option<*mut TempNode>,
    ) {
        let parent_content_width = if let Some(parent) = parent {
            let parent = unsafe { &mut *parent };
            parent.computed.width.content
        } else {
            window_size.width as f32
        };

        let ws = window_size;
        let pcw = parent_content_width;
        let padding = self.node.padding.compute_rect(pcw, ws);
        let border = self.node.border.compute_rect(pcw, ws);
        let margin = self.node.margin.compute_rect(pcw, ws);

        self.computed.padding = padding;
        self.computed.border = border;
        self.computed.margin = margin;

        let padding_border_horizontal = padding.horizontal() + border.horizontal();
        let padding_border_vertical = padding.vertical() + border.vertical();

        let mut set_content_box_width = || {
            self.computed.width.border += padding_border_horizontal;
            self.computed.width.total += padding_border_horizontal + margin.horizontal();
        };

        let mut set_content_box_height = || {
            self.computed.height.border += padding_border_vertical;
            self.computed.height.total += padding_border_vertical + margin.vertical();
        };

        if self.node.box_sizing == BoxSizing::ContentBox {
            set_content_box_width();
            set_content_box_height();
        } else {
            if self.node.width == Val::Auto {
                set_content_box_width();
            } else {
                self.computed.width.content -= padding_border_horizontal;
                self.computed.width.total += margin.vertical();

                if self.computed.width.content < 0.0 {
                    self.computed.width.border += self.computed.width.content.abs();
                    self.computed.width.total += self.computed.width.content.abs();
                    self.computed.width.content = 0.0;
                }
            }

            if self.node.height == Val::Auto {
                set_content_box_height();
            } else {
                self.computed.height.content -= padding_border_vertical;
                self.computed.height.total += margin.vertical();

                if self.computed.height.content < 0.0 {
                    self.computed.height.border += self.computed.height.content.abs();
                    self.computed.height.total += self.computed.height.content.abs();
                    self.computed.height.content = 0.0;
                }
            }
        }
    }

    /// Computes the row and column gaps
    /// Traversal: BOTTOM UP
    fn compute_gaps(&mut self, window_size: PhysicalSize<u32>) {
        let mut total_width_diff = 0.0;
        let mut max_width_diff: f32 = 0.0;
        let mut total_height_diff = 0.0;
        let mut max_height_diff: f32 = 0.0;

        for child in &mut self.children {
            let original_width = child.computed.width.content;
            let original_height = child.computed.height.content;

            child.compute_gaps(window_size);

            let width_diff = child.computed.width.content - original_width;
            let height_diff = child.computed.height.content - original_height;

            let width_growth = (child.computed.width.total - self.computed.width.content).max(0.0);
            let height_growth =
                (child.computed.height.total - self.computed.height.content).max(0.0);

            total_width_diff += width_diff;
            max_width_diff = max_width_diff.max(width_growth);
            total_height_diff += height_diff;
            max_height_diff = max_height_diff.max(height_growth);
        }

        // adjust size based on children growth
        if self.node.display == Display::Flex {
            if self.node.width == Val::Auto {
                match self.node.flex_direction.is_row() {
                    true => self.computed.width.add(total_width_diff),
                    false => self.computed.width.add(max_width_diff),
                }
            }

            if self.node.height == Val::Auto {
                match self.node.flex_direction.is_row() {
                    true => self.computed.height.add(max_height_diff),
                    false => self.computed.height.add(total_height_diff),
                }
            }
        }

        if self.node.display == Display::Grid {
            unimplemented!("Grid gaps");
        }

        // compute gaps
        if self.node.display == Display::Grid || self.node.display == Display::Flex {
            self.computed.column_gap = self
                .node
                .column_gap
                .compute_val(self.computed.width.content, window_size);
            self.computed.row_gap = self
                .node
                .row_gap
                .compute_val(self.computed.height.content, window_size);
        }
        if self.node.display == Display::Grid || self.node.is_flex_row() {
            self.computed.base_width +=
                self.computed.column_gap * self.children.len().saturating_sub(1) as f32;
        }

        // apply gaps
        if self.node.display == Display::Flex {
            let gaps_num = self.children.len().saturating_sub(1) as f32;

            if self.node.width == Val::Auto && self.node.flex_direction.is_row() {
                self.computed.width.add(self.computed.column_gap * gaps_num);
            }

            if self.node.height == Val::Auto && self.node.flex_direction.is_column() {
                self.computed.height.add(self.computed.row_gap * gaps_num);
            }
        }

        if self.node.display == Display::Grid {
            if self.node.width == Val::Auto {
                let gaps_num = self.node.grid_template_columns.len().saturating_sub(1) as f32;
                self.computed.width.add(self.computed.column_gap * gaps_num);
            }

            if self.node.height == Val::Auto {
                let gaps_num = self.node.grid_template_rows.len().saturating_sub(1) as f32;
                self.computed.height.add(self.computed.row_gap * gaps_num);
            }
        }

        self.constrain_to_width();
        self.constrain_to_height();
    }

    /// Applies min / max constraints, computes box sizing.
    /// In addition to that it also resolves the text color.
    /// Traversal: TOP DOWN
    fn apply_constraints(&mut self, window_size: PhysicalSize<u32>, parent: Option<*mut TempNode>) {
        self.compute_min_max(window_size, parent);
        self.compute_box_sizing(window_size, parent);

        for child in &mut self.children {
            child.apply_constraints(window_size, parent);

            // text color
            child.computed.color = child.node.color.unwrap_or(self.computed.color);
        }
    }

    /// Computes the translations (screen space position)
    /// Traversal: BOTTOM UP
    fn compute_translation(&mut self) {
        // apply box offset
        let self_offset_x = self.computed.border.left + self.computed.padding.left;
        let self_offset_y = self.computed.border.top + self.computed.padding.top;

        self.transform.translation = Vec3::new(
            self.computed.margin.left,
            self.computed.margin.top,
            self.computed.z_index as f32,
        );

        match self.node.display {
            Display::None => {}
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
            }
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
                        offset_x += child.computed.width.total + self.computed.column_gap;
                    } else {
                        offset_y += child.computed.height.total + self.computed.row_gap;
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
            FlexDirection::Row | FlexDirection::RowReverse => self
                .children
                .iter()
                .map(|child| {
                    let diff = self.computed.height.content - child.computed.height.total;
                    let offset = match self.node.align_items {
                        AlignItems::FlexStart => 0.0,
                        AlignItems::FlexEnd => diff,
                        AlignItems::Center => diff / 2.0,
                        AlignItems::Stretch => 0.0,
                    };
                    Vec3::new(0.0, offset, 0.0)
                })
                .collect(),
            FlexDirection::Column | FlexDirection::ColumnReverse => self
                .children
                .iter()
                .map(|child| {
                    let diff = self.computed.width.content - child.computed.width.total;
                    let offset = match self.node.align_items {
                        AlignItems::FlexStart => 0.0,
                        AlignItems::FlexEnd => diff,
                        AlignItems::Center => diff / 2.0,
                        AlignItems::Stretch => 0.0,
                    };
                    Vec3::new(offset, 0.0, 0.0)
                })
                .collect(),
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

        let content_size = self
            .children
            .iter()
            .fold(gaps_num * computed_gap, |acc, child| {
                acc + if is_row {
                    child.computed.width.total
                } else {
                    child.computed.height.total
                }
            });

        let offset = if is_row {
            self.computed.width.content
        } else {
            self.computed.height.content
        } - content_size;

        match self.node.justify_content {
            JustifyContent::FlexStart => self.offsets_from(Vec3::ZERO),
            JustifyContent::FlexEnd => self.offsets_from(new_vec3(offset)),
            JustifyContent::Center => self.offsets_from(new_vec3(offset / 2.0)),
            JustifyContent::SpaceBetween => {
                let offset = offset.max(0.0);
                let between_gap = offset / gaps_num;
                self.children
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let gap = if i == 0 { 0.0 } else { between_gap * i as f32 };
                        new_vec3(gap)
                    })
                    .collect()
            }
            JustifyContent::SpaceAround => {
                let offset = offset.max(0.0);
                let around_gap = offset / (gaps_num + 1.0);
                self.children
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let gap = (around_gap * i as f32) + around_gap / 2.0;
                        new_vec3(gap)
                    })
                    .collect()
            }
            JustifyContent::SpaceEvenly => {
                let offset = offset.max(0.0);
                let even_gap = offset / (gaps_num + 2.0);
                self.children
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let gap = even_gap * (i + 1) as f32;
                        new_vec3(gap)
                    })
                    .collect()
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

    /// Resolves text wrapping and adjusts auto-sized elements
    /// Traversal: TOP DOWN
    fn resolve_text_wrap(&mut self, font_system: &mut FontSystem) {
        if self.node.display == Display::None {
            return;
        }

        // wrap text
        if let Some(ref mut rae) = self.text_rae {
            let max_width = self.computed.width.content;
            let prev_heigh = rae.height();
            rae.set_size(font_system, Some(max_width), None);
            let new_height = rae.height();

            // adjust height
            let diff = new_height - prev_heigh;
            if self.node.height == Val::Auto {
                self.computed.height.add(diff);
                self.constrain_to_height();
            }
        }

        // adjust width for flex row shrink
        if self.node.is_flex_row() {
            let total = self
                .children
                .iter()
                .fold(0.0, |acc, child| acc + child.computed.width.total);
            let overflow = self.computed.width.content - total;
            let diff = overflow / self.children.len() as f32;

            if diff < 0.0 {
                for child in &mut self.children {
                    child.computed.width.add(diff);
                    child.constrain_to_width();
                }
            }
        }

        for child in &mut self.children {
            // TODO: take box sizing into account ?
            // fit x-overflowing element which can be sized to parent
            if matches!(child.node.width, Val::Auto)
                && child.computed.width.total > self.computed.width.content
                && child.computed.base_width <= self.computed.width.content
            {
                // shrink width to parent width
                // byproduct is wrapping text to fit screen-width
                let diff = child.computed.width.total - self.computed.width.content;
                child.computed.width.add(-diff);
                child.constrain_to_width();
            }

            child.resolve_text_wrap(font_system);
        }
    }

    /// clamp one node in respect to min / max width
    fn constrain_to_width(&mut self) {
        let (min_diff, max_diff) = if self.node.box_sizing == BoxSizing::ContentBox {
            let min = (self.computed.min_width - self.computed.width.content).max(0.0);
            let max = (self.computed.max_width - self.computed.width.content).min(0.0);
            (min, max)
        } else {
            let min = (self.computed.min_width - self.computed.width.border).max(0.0);
            let max = (self.computed.max_width - self.computed.width.border).min(0.0);
            (min, max)
        };

        if min_diff > 0.0 {
            self.computed.width.add(min_diff);
        } else {
            self.computed.width.add(max_diff);
        }
    }

    /// clamp one node in respect to min / max height
    fn constrain_to_height(&mut self) {
        let (min_diff, max_diff) = if self.node.box_sizing == BoxSizing::ContentBox {
            let min = (self.computed.min_height - self.computed.height.content).max(0.0);
            let max = (self.computed.max_height - self.computed.height.content).min(0.0);
            (min, max)
        } else {
            let min = (self.computed.min_height - self.computed.height.border).max(0.0);
            let max = (self.computed.max_height - self.computed.height.border).min(0.0);
            (min, max)
        };

        if min_diff > 0.0 {
            self.computed.height.add(min_diff);
        } else {
            self.computed.height.add(max_diff);
        }
    }

    /// Recalculates auto-sized elements
    /// Traversal: BOTTOM UP
    fn fit_auto_size(&mut self) {
        if self.children.is_empty() {
            return;
        }

        let mut total_width = 0.0;
        let mut total_height = 0.0;

        let mut max_width = 0.0f32;
        let mut max_height = 0.0f32;

        for child in &mut self.children {
            child.fit_auto_size();

            total_width += child.computed.width.total;
            total_height += child.computed.height.total;

            max_width = max_width.max(child.computed.width.total);
            max_height = max_height.max(child.computed.height.total);
        }

        let gaps_num = self.children.len().saturating_sub(1) as f32;

        if self.node.width == Val::Auto {
            let width = match (self.node.display, self.node.flex_direction.is_row()) {
                (Display::Flex, true) => total_width + self.computed.column_gap * gaps_num,
                (Display::Block, _) | (Display::Flex, false) => max_width,
                (Display::None, _) => return,
                (Display::Grid, _) => unimplemented!("Grid fit auto size"),
            };

            let growth = width - self.computed.width.content;
            self.computed.width.add(growth);
        }

        if self.node.height == Val::Auto {
            let height = match (self.node.display, self.node.flex_direction.is_row()) {
                (Display::Flex, true) => max_height,
                (Display::Block, _) => total_height,
                (Display::Flex, false) => total_height + self.computed.row_gap * gaps_num,
                (Display::None, _) => return,
                (Display::Grid, _) => unimplemented!("Grid fit auto size"),
            };

            let growth = height - self.computed.height.content;
            self.computed.height.add(growth);
        }
    }

    /// Resolves flex grow and shrink
    /// Traverse: TOP DOWN
    fn resolve_flex(&mut self) {
        if self.node.display == Display::Grid {
            unimplemented!("Grid flex grow and shrink");
        }

        if self.node.display != Display::Flex {
            return;
        }

        for child in &mut self.children {
            if child.node.width == Val::Auto
                && self.node.align_items == AlignItems::Stretch
                && (self.node.is_flex_column() || self.node.display == Display::Block)
            {
                let diff = self.computed.width.content - child.computed.width.total;
                child.computed.width.add(diff);
                child.constrain_to_width();
            }

            if child.node.height == Val::Auto && self.node.does_stretch_height() {
                let diff = self.computed.height.content - child.computed.height.total;
                child.computed.height.add(diff);
                child.constrain_to_height();
            }

            child.resolve_flex();
        }
    }

    /// Recalculates percent sized elements after finalized parents
    /// Traversal: TOP DOWN
    fn recalculate_percent_size(&mut self) {
        if self.node.display == Display::None {
            return;
        }

        for child in &mut self.children {
            if !self.node.is_flex_row()
                && let Val::Percent(val) = child.node.width
            {
                let new_width = self.computed.width.content * val / 100.0;
                let diff = new_width - child.computed.width.total;
                child.computed.width.add(diff);
                child.constrain_to_width();
            }

            if !self.node.is_flex_column()
                && let Val::Percent(val) = child.node.height
            {
                let new_height = self.computed.height.content * val / 100.0;
                let diff = new_height - child.computed.height.total;
                child.computed.height.add(diff);
                child.constrain_to_height();
            }

            child.recalculate_percent_size();
        }
    }
}

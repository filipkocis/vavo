use glam::Vec3;
use glyphon::FontSystem;

use crate::{prelude::*, render_assets::RenderAssets, ui::text::TextBuffer};

use super::build_temp::{nodes_to_temp_graph, TempNode};

/// Post update system to compute ui nodes and update their transforms
pub fn compute_nodes_and_transforms(ctx: &mut SystemsContext, mut q: Query<()>) {
    let mut temp_nodes = nodes_to_temp_graph(ctx, &mut q);

    if temp_nodes.is_empty() {
        return;
    }

    let mut text_buffers = ctx.resources.get_mut::<RenderAssets<TextBuffer>>().expect("TextBuffer render assets not found");
    compute_z_index_for_nodes(ctx, &mut text_buffers, &mut temp_nodes, &mut 0);
    
    for temp_node in &mut temp_nodes {
        temp_node.compute(None, ctx);
        temp_node.compute_translation(Vec3::ZERO, ctx);
    }
}

/// Sorts nodes by z_index and then computes the z_index with depth first search.
/// Starts with layer 0, increments by 1 for each node.
///
/// # Note
/// Doesn't handle `position: absolute`.
///
/// # Important
/// When setting z_index on text, it will recreate the text buffer render asset.
pub fn compute_z_index_for_nodes(
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

        compute_z_index_for_nodes(ctx, text_buffers, &mut node.children, layer);
    }
}

impl TempNode<'_> {
    /// Computes the translation of self, then of its children.
    /// Offset is relative to parent, includes parents padding, border and children offset.
    ///
    /// # Note
    /// Requires node and z_index to be computed
    pub fn compute_translation(&mut self, offset: Vec3, ctx: &mut SystemsContext) {
        let padding = &self.computed.padding;
        let border = &self.computed.border;
        let mut translation = offset;

        // apply margin
        translation.x += self.computed.margin.left;
        translation.y += self.computed.margin.top;
        translation.z = self.computed.z_index as f32;

        // apply self offset
        let mut child_offset = Vec3::new(
            padding.left + border.left,
            padding.top + border.top,
            0.0,
        );

        match self.node.display {
            Display::Flex => {
                let justify_content_offsets = self.justify_content_offsets();
                let align_items_offsets = self.align_items_offsets();

                for (i, child) in self.children.iter_mut().enumerate() {
                    let justify_content_offset = justify_content_offsets[i];
                    let align_items_offset = align_items_offsets[i];
                    child.compute_translation(child_offset + justify_content_offset + align_items_offset, ctx);

                    match self.node.flex_direction {
                        FlexDirection::Row | FlexDirection::RowReverse => {
                            child_offset.x += 
                                child.computed.width.total + 
                                self.computed.column_gap;
                        },
                        FlexDirection::Column | FlexDirection::ColumnReverse => {
                            child_offset.y += 
                                child.computed.height.total + 
                                self.computed.row_gap;
                        }
                    }
                }
            },
            Display::Block => {
                for child in &mut self.children {
                    child.compute_translation(child_offset, ctx);
                    child_offset.y += child.computed.height.total; 
                } 
            },
            Display::Grid => {
                todo!("Grid layout")
            },
            Display::None => {
                // keep translation, should not be rendered
            }
        };

        self.transform.translation = translation;
    }

    /// Returns the offsets for `align-items` for each child node.
    /// `result.len() == self.children.len()`
    ///
    /// Only the main cross-axis field is used in a flex container, otherwise fields are 0.0.
    pub fn align_items_offsets(&self) -> Vec<Vec3> {
        if self.node.display != Display::Flex {
            return (0..self.children.len()).map(|_| Vec3::ZERO).collect::<Vec<_>>()
        }

        match self.node.flex_direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                self.children.iter().map(|child| {
                    let diff = self.computed.height.content - child.computed.height.total;
                    let offset = match self.node.align_items {
                        AlignItems::FlexStart => 0.0,
                        AlignItems::FlexEnd => diff,
                        AlignItems::Center => diff / 2.0,
                        AlignItems::Stretch => 0.0, // TODO: children are stretched during computation
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
                        AlignItems::Stretch => 0.0, // TODO: children are stretched during computation
                    };
                    Vec3::new(offset, 0.0, 0.0)
                }).collect()
            }
        }
    }

    /// Returns the offsets for `justify-content` for each child node.
    /// `result.len() == self.children.len()`
    ///
    /// Only the main axis field is used in a flex container, otherwise fields are 0.0.
    pub fn justify_content_offsets(&self) -> Vec<Vec3> {
        let offsets_from = |v: Vec3| {
            (0..self.children.len()).map(|_| v).collect::<Vec<_>>()
        };

        if self.node.display != Display::Flex {
            return offsets_from(Vec3::ZERO); 
        }

        let gaps_num = (self.children.len() as isize - 1).max(0) as f32;

        // TODO: refactor into one function to not duplicate code
        match self.node.flex_direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                let content_width = self.children.iter().fold(gaps_num * self.computed.column_gap, |acc, child| 
                    acc + child.computed.width.total
                );
                let offset = self.computed.width.content - content_width;

                match self.node.justify_content {
                    JustifyContent::FlexStart => return offsets_from(Vec3::ZERO),
                    JustifyContent::FlexEnd => return offsets_from(Vec3::new(offset, 0.0, 0.0)),
                    JustifyContent::Center => return offsets_from(Vec3::new(offset / 2.0, 0.0, 0.0)), 
                    JustifyContent::SpaceBetween => {
                        let offset = offset.max(0.0);
                        let between_gap = offset / gaps_num;
                        self.children.iter().enumerate().map(|(i, _)| {
                            let gap = if i == 0 { 0.0 } else { between_gap * i as f32 };
                            Vec3::new(gap, 0.0, 0.0)
                        }).collect()
                    },
                    JustifyContent::SpaceAround => {
                        let offset = offset.max(0.0);
                        let around_gap = offset / (gaps_num + 1.0);  
                        self.children.iter().enumerate().map(|(i, _)| {
                            let gap = (around_gap * i as f32) + around_gap / 2.0;
                            Vec3::new(gap, 0.0, 0.0)
                        }).collect()
                    }
                    JustifyContent::SpaceEvenly => {
                        let offset = offset.max(0.0);
                        let even_gap = offset / (gaps_num + 2.0);  
                        self.children.iter().enumerate().map(|(i, _)| {
                            let gap = even_gap * (i + 1) as f32;
                            Vec3::new(gap, 0.0, 0.0)
                        }).collect()
                    }
                }
            },
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                let content_height = self.children.iter().fold(gaps_num * self.computed.row_gap, |acc, child| 
                    acc + child.computed.height.total
                );
                let offset = self.computed.height.content - content_height;

                match self.node.justify_content {
                    JustifyContent::FlexStart => return offsets_from(Vec3::ZERO),
                    JustifyContent::FlexEnd => return offsets_from(Vec3::new(0.0, offset, 0.0)),
                    JustifyContent::Center => return offsets_from(Vec3::new(0.0, offset / 2.0, 0.0)), 
                    JustifyContent::SpaceBetween => {
                        let offset = offset.max(0.0);
                        let between_gap = offset / gaps_num;
                        self.children.iter().enumerate().map(|(i, _)| {
                            let gap = if i == 0 { 0.0 } else { between_gap * i as f32 };
                            Vec3::new(0.0, gap, 0.0)
                        }).collect()
                    },
                    JustifyContent::SpaceAround => {
                        let offset = offset.max(0.0);
                        let around_gap = offset / (gaps_num + 1.0);  
                        self.children.iter().enumerate().map(|(i, _)| {
                            let gap = (around_gap * i as f32) + around_gap / 2.0;
                            Vec3::new(0.0, gap, 0.0)
                        }).collect()
                    }
                    JustifyContent::SpaceEvenly => {
                        let offset = offset.max(0.0);
                        let even_gap = offset / (gaps_num + 2.0);
                        self.children.iter().enumerate().map(|(i, _)| {
                            let gap = even_gap * (i + 1) as f32;
                            Vec3::new(0.0, gap, 0.0)
                        }).collect()
                    }
                }
            },
        }
    }
}

impl TempNode<'_> {
    /// Computes the node and its children
    pub fn compute(&mut self, parent: Option<*const TempNode>, ctx: &mut SystemsContext) {
        // width and box calculations
        let width = self.compute_width_and_box(parent, ctx); 
        self.computed.width = width;

        // height depends on width and its calculations
        let height = self.compute_height(parent, ctx); 
        self.computed.height = height;

        // text color
        self.computed.color = match self.node.color {
            Some(color) => color,
            None => match parent {
                Some(parent) => unsafe { &*parent }.computed.color,
                None => color::BLACK,
            }
        };

        // compute grid, TODO: grid layout
        let template_columns = self.node.grid_template_columns.iter().map(|val| val.compute_val(width.content, ctx));
        let template_rows = self.node.grid_template_rows.iter().map(|val| val.compute_val(height.content, ctx));
        self.computed.grid_template_columns = template_columns.collect();
        self.computed.grid_template_rows = template_rows.collect();

        // compute children
        let self_as_parent = self as *const TempNode;
        for child in self.children.iter_mut() {
            child.compute(Some(self_as_parent), ctx);
        }
    }

    /// Computes the min and max sizes.
    fn compute_min_max_size(
        &self, 
        ctx: &SystemsContext, 
        parent: Option<*const TempNode>, 
        parent_content: f32, 
        is_width: bool
    ) -> (f32, f32) {
        let map_max = |val: Val| if val == Val::Auto {
            // use parent content size as max, or self computed size if parent is auto
            if let Some(parent) = parent {
                if unsafe { &*parent }.node.width == Val::Auto {
                    f32::INFINITY
                } else {
                    parent_content
                }
            } else {
                parent_content
            }
        } else {
            val.compute_val(parent_content, ctx)
        };

        let map_min = |val: Val| if val == Val::Auto {
            0.0
        } else {
            val.compute_val(parent_content, ctx)
        };

        let (min, max) = if is_width {
            (
                map_min(self.node.min_width),
                map_max(self.node.max_width),
            )
        } else {
            (
                map_min(self.node.min_height),
                map_max(self.node.max_height),
            )
        };

        (min, max)
    }

    /// Computes the sizes for `ComputedBox` based on `box-sizing`
    fn compute_box_sizing(&self, computed_size: f32, is_width: bool) -> (f32, f32, f32) {
        // TODO: check how percentage sizes should work, perhaps they should always use `border-box`

        let (size, padding_border, margin, stretch, min, max) = if is_width {
            (
                self.node.width,
                self.computed.padding.horizontal() + self.computed.border.horizontal(),
                self.computed.margin.horizontal(),
                self.computed.stretch_width,
                self.computed.min_width, self.computed.max_width,
            )
        } else {
            (
                self.node.height,
                self.computed.padding.vertical() + self.computed.border.vertical(),
                self.computed.margin.vertical(),
                self.computed.stretch_height,
                self.computed.min_height, self.computed.max_height,
            )
        };

        let mut content;
        let mut border;
        let total;
        
        if (self.node.box_sizing == BoxSizing::ContentBox || // box-sizing: content-box 
            size == Val::Auto) && // size: auto -> box-sizing has no effect
            !stretch { // border-box if stretched
        // if self.node.box_sizing == BoxSizing::ContentBox && // box-sizing: content-box 
        //     // size == Val::Auto) && // size: auto -> box-sizing has no effect
        //     !stretch { // border-box if stretched
            content = computed_size;
            border = content + padding_border;
            total = border + margin;
        // TODO: fix overflow for auto elements, maybe it needs to be done here ? 
        // } else if size == Val::Auto {
        //     if computed_size + padding_border <= max {
        //         content = computed_size;
        //         border = content + padding_border;
        //         total = border + margin;
        //     } else {
        //         border = computed_size;
        //         content = border - padding_border;
        //
        //         if content < 0.0 {
        //             // if padding + border is greater than content width
        //             let overflow = content.abs();
        //             border += overflow;
        //             content = 0.0;
        //         }
        //
        //         total = border + margin;
        //     }
        } else { // box-sizing: border-box
            border = computed_size;
            content = border - padding_border;

            if content < 0.0 {
                // if padding + border is greater than content width
                let overflow = content.abs();
                border += overflow;
                content = 0.0;
            }

            total = border + margin;
        }

        (content, border, total)
    }

    /// Returns the computed height of the node
    ///
    /// # Note
    /// Requires padding, border and margin to be computed.
    /// Text buffer width is required to be set in order to get correct auto height.
    fn compute_height(&mut self, parent: Option<*const TempNode>, ctx: &mut SystemsContext) -> ComputedBox {
        let screen_size = ctx.renderer.size();
        let parent_height = match parent {
            Some(parent) => unsafe { &*parent }.computed.height.content,
            None => screen_size.height as f32,
        };

        // min max
        let (min, max) = self.compute_min_max_size(ctx, parent, parent_height, false);
        self.computed.min_height = min;
        self.computed.max_height = max;

        let mut computed_height = match self.node.height {
            Val::Percent(percent) => parent_height * percent / 100.0,
            Val::Vw(vw) => screen_size.width as f32 * vw / 100.0,
            Val::Vh(vh) => screen_size.height as f32 * vh / 100.0,
            Val::Px(px) => px,
            Val::Rem(rem) => rem * 16.0,
            Val::Auto => match self.computed.stretch_height {
                true => parent_height,
                false => self.compute_auto_height(parent, ctx),
            }
        };

        if let Some(parent) = parent {
            let p = unsafe { &*parent };
            if p.node.display == 
                Display::Flex && p.node.flex_direction.is_row() && 
                self.node.height == Val::Auto && p.node.align_items == AlignItems::Stretch {
                    if p.node.height != Val::Auto {
                        computed_height = parent_height;
                    }
                    self.computed.stretch_height = true;
            } else {
                self.computed.stretch_height = false;
            }
        }

        let computed_height = computed_height.min(max).max(min);

        // compute box sizing
        let (content, border, total) = self.compute_box_sizing(computed_height, false);

        // set text buffer height
        if let Some(ref text_rae) = self.text_rae {
            let mut font_system = ctx.resources.get_mut::<FontSystem>().expect("FontSystem resource not found");
            text_rae.set_size(&mut font_system, Some(self.computed.width.content), Some(content));
        }

        ComputedBox {
            content,
            border,
            total,
        }
    }

    /// Returns the computed height of the node when the height is set to auto
    ///
    /// # Note
    /// Percentage values in deps do not work correctly.
    fn compute_auto_height(&mut self, parent: Option<*const TempNode>, ctx: &mut SystemsContext) -> f32 {
        let mut height = 0.0;
        let flex_direction = self.node.flex_direction;

        // accumulate children heights
        let mut max_height: f32 = 0.0;
        let self_as_parent = self as *const TempNode;
        for child in &mut self.children {
            let h = child.compute_height(Some(self_as_parent), ctx).total;
            height += h;
            max_height = max_height.max(h);
        }

        match self.node.display {
            Display::Flex => {
                match flex_direction {
                    FlexDirection::Column | FlexDirection::ColumnReverse => {
                        // accumulate flex gap
                        let len = (self.children.len() as isize - 1).max(0) as f32;
                        height += len * self.computed.row_gap
                    },
                    FlexDirection::Row | FlexDirection::RowReverse => {
                        height = max_height;
                    }
                }
            },
            Display::Block => {
                // keep accumulated height
            },
            Display::Grid => {
                todo!("grid layout")
            },
            Display::None => {
                height = 0.0;
            }
        }

        let (parent_width_auto, parent_height_auto) = parent.map(|p| {
            let p = unsafe { &*p }; 
            (p.node.width == Val::Auto, p.node.height == Val::Auto)
        }).unwrap_or((false, false)); // if no parent, screen size is fixed
        // text height, if label is present first compute the width to get correct height from text
        // wrrapping, this may be needed if parent has fixed width but auto height
        if self.text_rae.is_some() && !parent_width_auto && parent_height_auto {
            self.compute_width_and_box(parent, ctx);
        }
        let text_height = self.text_rae.as_ref()
            .map(|rae| rae.height())
            .unwrap_or_default();

        // return final height
        height.max(text_height)
    }

    /// Returns the computed width box of the node
    ///
    /// # Note
    /// Requires margin to be computed.
    /// Will also compute and set margin, padding, border, column_gap, row_gap.
    /// Based on text css it will also set and recreate its text buffer width.
    fn compute_width_and_box(&mut self, parent: Option<*const TempNode>, ctx: &mut SystemsContext) -> ComputedBox {
        let screen_size = ctx.renderer.size();
        let parent_width = match parent {
            Some(parent) => unsafe { &*parent }.computed.width.content,
            None => screen_size.width as f32,
        };

        // column gap, row gap
        // TODO: does not work for percentage values
        let column_gap = self.node.column_gap.compute_val(0.0, ctx);
        let row_gap = self.node.row_gap.compute_val(0.0, ctx);
        self.computed.column_gap = column_gap;
        self.computed.row_gap = row_gap;

        // min max
        let (min, max) = self.compute_min_max_size(ctx, parent, parent_width, true);
        self.computed.min_width = min;
        self.computed.max_width = max;

        let mut computed_width = match self.node.width {
            Val::Percent(percent) => parent_width * percent / 100.0,
            Val::Vw(vw) => screen_size.width as f32 * vw / 100.0,
            Val::Vh(vh) => screen_size.height as f32 * vh / 100.0,
            Val::Px(px) => px,
            Val::Rem(rem) => rem * 16.0,
            Val::Auto => match self.computed.stretch_width {
                true => parent_width,
                false => self.compute_auto_width(parent, ctx),
            }
        };

        if let Some(parent) = parent {
            let p = unsafe { &*parent };
            if p.node.display == 
                Display::Flex && p.node.flex_direction.is_column() && 
                self.node.width == Val::Auto && p.node.align_items == AlignItems::Stretch {
                    if p.node.width != Val::Auto {
                        computed_width = parent_width;
                    }
                    self.computed.stretch_width = true;
            } else {
                self.computed.stretch_width = false;
            }
        }

        let computed_width = computed_width.min(max).max(min);

        // margin
        let margin = self.node.margin.compute_rect(parent_width, ctx);
        self.computed.margin = margin;

        // padding
        let padding = self.node.padding.compute_rect(parent_width, ctx);
        self.computed.padding = padding;

        // border
        // TODO: does not work for percentage values in children, because that would require this
        // to be calculated before `content` width
        let border = self.node.border.compute_rect(computed_width, ctx);
        self.computed.border = border;

        // compute box sizing
        let (content, border, total) = self.compute_box_sizing(computed_width, true); 

        // set text buffer width
        if let Some(ref text_rae) = self.text_rae {
            let mut font_system = ctx.resources.get_mut::<FontSystem>().expect("FontSystem resource not found");
            text_rae.set_size(&mut font_system, Some(content), None);
        }

        ComputedBox {
            content,
            border,
            total, 
        }
    }

    /// Returns the computed width of the node when the width is set to auto
    ///
    /// # Note
    /// Percentage values in deps do not work correctly.
    fn compute_auto_width(&mut self, parent: Option<*const TempNode>, ctx: &mut SystemsContext) -> f32 {
        let mut width = 0.0;
        let flex_direction = self.node.flex_direction;

        // accumulate children widths
        let mut max_width: f32 = 0.0;
        let self_as_parent = self as *const TempNode;
        for child in &mut self.children {
            let w = child.compute_width_and_box(Some(self_as_parent), ctx).total;
            width += w;
            max_width = max_width.max(w);
        }

        match self.node.display {
            Display::Flex => {
                match flex_direction {
                    FlexDirection::Row | FlexDirection::RowReverse => {
                        // accumulate flex gap
                        let len = (self.children.len() as isize - 1).max(0) as f32;
                        width += len * self.computed.column_gap
                    },
                    FlexDirection::Column | FlexDirection::ColumnReverse => {
                        width = max_width;
                    }
                }
            },
            Display::Block => {
                width = max_width;             
            },
            Display::Grid => {
                todo!("grid layout")
            },
            Display::None => {
                width = 0.0;
            }
        }

        // text width, if label is present
        let text_width = self.text_rae.as_ref()
            .map(|rae| {
                let mut font_system = ctx.resources.get_mut::<FontSystem>().expect("FontSystem resource not found");
                let max_screen = ctx.renderer.size().width as f32;

                // max width for text wrapping, doesnt take padding or border into account
                let max_width = parent.map(|p| { 
                        let p = unsafe { &*p };
                        if p.node.width == Val::Auto {
                            p.computed.max_width
                        } else {
                            p.computed.width.content
                        }
                    })
                    .unwrap_or(max_screen);

                // first reset size to get accurate auto width
                rae.set_size(&mut font_system, Some(max_width), None);
                rae.width()
            })
            .unwrap_or_default();
        
        // return final width
        width.max(text_width)
    }
}

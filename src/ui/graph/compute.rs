use glam::Vec3;

use crate::{prelude::*, ui::node::{ComputedBox, ComputedRect, Display, FlexDirection, Rect, Val}};

use super::build_temp::{nodes_to_temp_graph, TempNode};

/// Post update system to compute ui nodes and update their transforms
pub fn compute_nodes_and_transforms(ctx: &mut SystemsContext, mut q: Query<()>) {
    let mut temp_nodes = nodes_to_temp_graph(&mut q);

    compute_z_index_for_nodes(&mut temp_nodes, &mut 0);
    
    for temp_node in &mut temp_nodes {
        dbg!(&temp_node);
        temp_node.compute(None, ctx);
        temp_node.compute_translation(Vec3::ZERO, ctx);
    }
}

/// Sorts nodes by z_index and then computes the z_index with depth first search.
/// Starts with layer 0, increments by 1 for each node.
///
/// # Note
/// Doesn't handle `position: absolute`
pub fn compute_z_index_for_nodes(nodes: &mut Vec<TempNode>, layer: &mut usize) {
    nodes.sort_by(|a, b| a.node.z_index.cmp(&b.node.z_index)); 

    for node in nodes.iter_mut() {
        node.computed.z_index = *layer as i32;
        *layer += 1;

        compute_z_index_for_nodes(&mut node.children, layer);
    }
}

impl TempNode<'_> {
    /// Computes the translation of self, then of its children.
    /// Offset is relative to parent, includes parents padding, border and children offset.
    ///
    /// # Note
    /// Requires node and z_index to be computed
    pub fn compute_translation(&mut self, mut offset: Vec3, ctx: &mut SystemsContext) {
        let padding = &self.computed.padding;
        let border = &self.computed.border;
        let mut translation = offset;

        // apply margin
        translation.x += self.computed.margin.left;
        translation.y += self.computed.margin.top;

        // apply self offset
        offset.x += padding.left + border.left;
        offset.y += padding.top + border.top;

        match self.node.display {
            Display::Flex => {
                for child in &mut self.children {
                    child.compute_translation(offset, ctx);

                    match self.node.flex_direction {
                        FlexDirection::Row | FlexDirection::RowReverse => {
                            offset.x += 
                                child.computed.width.border + 
                                child.computed.margin.horizontal() +
                                child.computed.border.horizontal() +
                                self.computed.column_gap;
                        },
                        FlexDirection::Column | FlexDirection::ColumnReverse => {
                            offset.y += 
                                child.computed.height.border + 
                                child.computed.margin.vertical() +
                                child.computed.border.vertical() +
                                self.computed.row_gap;
                        }
                    }
                }
            },
            Display::Block => {
                for child in &mut self.children {
                    child.compute_translation(offset, ctx);

                    offset.y += 
                        child.computed.height.border + 
                        child.computed.margin.vertical() +
                        child.computed.border.vertical();
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
}

impl Val {
    pub fn compute_val(&self, parent: f32, ctx: &SystemsContext) -> f32 {
        let window_size = ctx.renderer.size();
        match self {
            Val::Auto => 0.0,
            Val::Px(val) => *val,
            Val::Rem(val) => *val * 16.0,
            Val::Percent(val) => parent * *val / 100.0,
            Val::Vw(val) => window_size.width as f32 * *val / 100.0,
            Val::Vh(val) => window_size.height as f32 * *val / 100.0,
        }
    }
}

impl Rect {
    /// Compute Rect fields based on parent width for padding and margin, self width for border
    pub fn compute_rect(&self, width: f32, ctx: &mut SystemsContext) -> ComputedRect {
        ComputedRect {
            left: self.left.compute_val(width, ctx),
            right: self.right.compute_val(width, ctx),
            top: self.top.compute_val(width, ctx),
            bottom: self.bottom.compute_val(width, ctx),
        }
    }
}

impl ComputedRect {
    /// Returns the horizontal sum of left and right
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Returns the vertical sum of top and bottom
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
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

    /// Returns the computed height of the node
    ///
    /// # Note
    /// Requires padding, border and margin to be computed
    fn compute_height(&mut self, parent: Option<*const TempNode>, ctx: &mut SystemsContext) -> ComputedBox {
        let screen_size = ctx.renderer.size();
        let parent_height = match parent {
            Some(parent) => unsafe { &*parent }.computed.height.content,
            None => screen_size.height as f32,
        };

        let content = match self.node.height {
            Val::Percent(percent) => parent_height * percent / 100.0,
            Val::Vw(vw) => screen_size.width as f32 * vw / 100.0,
            Val::Vh(vh) => screen_size.height as f32 * vh / 100.0,
            Val::Px(px) => px,
            Val::Rem(rem) => rem * 16.0,
            Val::Auto => self.compute_auto_height(parent, ctx),
        };

        let border = content + self.computed.padding.vertical() + self.computed.border.vertical(); 
        let total = border + self.computed.margin.vertical();

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
        
        // return final height
        height
    }

    /// Returns the computed width box of the node
    ///
    /// # Note
    /// Requires margin to be computed
    /// Will also compute and set margin, padding, border, column_gap, row_gap
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

        let content = match self.node.width {
            Val::Percent(percent) => parent_width * percent / 100.0,
            Val::Vw(vw) => screen_size.width as f32 * vw / 100.0,
            Val::Vh(vh) => screen_size.height as f32 * vh / 100.0,
            Val::Px(px) => px,
            Val::Rem(rem) => rem * 16.0,
            Val::Auto => self.compute_auto_width(parent, ctx),
        };

        // margin
        let margin = self.node.margin.compute_rect(parent_width, ctx);
        self.computed.margin = margin;

        // padding
        let padding = self.node.padding.compute_rect(parent_width, ctx);
        self.computed.padding = padding;

        // border
        // TODO: does not work for percentage values in children, because that would require this
        // to be calculated before `content` width
        let border = self.node.border.compute_rect(content, ctx);
        self.computed.border = border;

        // border box
        let border = content + self.computed.padding.horizontal() + self.computed.border.horizontal();
        
        ComputedBox {
            content,
            border,
            total: border + self.computed.margin.horizontal(), 
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
        
        // return final width
        width 
    }
}

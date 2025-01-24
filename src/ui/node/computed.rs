use crate::{prelude::Color, system::SystemsContext};

use super::{UiRect, Val};

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

#[derive(Default, Debug, Clone, Copy)]
pub struct ComputedUiRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl ComputedUiRect {
    /// Returns the horizontal sum of left and right
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Returns the vertical sum of top and bottom
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl UiRect {
    /// Compute Rect fields based on parent width for padding and margin, self width for border
    pub fn compute_rect(&self, width: f32, ctx: &mut SystemsContext) -> ComputedUiRect {
        ComputedUiRect {
            left: self.left.compute_val(width, ctx),
            right: self.right.compute_val(width, ctx),
            top: self.top.compute_val(width, ctx),
            bottom: self.bottom.compute_val(width, ctx),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ComputedBox {
    /// Content size, in css:
    /// `box-sizing: content-box`
    pub content: f32,
    /// Content + padding + border size, in css:
    /// `box-sizing: border-box`
    pub border: f32,
    /// Total size, including margin, no css equivalent
    pub total: f32,
}

impl ComputedBox {
    /// Returns the content offset relative to the border-box
    /// `(border - content) / 2 = offset`
    pub fn offset(&self) -> f32 {
        (self.border - self.content) / 2.0
    }
}

#[derive(Default, Debug, Clone)]
pub struct ComputedNode {
    pub color: Color,
    pub z_index: i32,

    pub grid_template_columns: Vec<f32>,
    pub grid_template_rows: Vec<f32>,

    pub column_gap: f32,
    pub row_gap: f32,

    pub padding: ComputedUiRect,
    pub margin: ComputedUiRect,
    pub border: ComputedUiRect,

    pub width: ComputedBox,
    pub min_width: f32,
    pub max_width: f32,
    pub height: ComputedBox,
    pub min_height: f32,
    pub max_height: f32,

    /// True if width is auto and parent is a column flexbox with stretch alignment
    pub stretch_width: bool,
    /// True if height is auto and parent is a row flexbox with stretch alignment
    pub stretch_height: bool,

    // /// Scale used by children to fit in a row flexbox container
    // pub width_scale: f32,
    // /// Scale used by children to fit in a column flexbox container
    // pub height_scale: f32,
}

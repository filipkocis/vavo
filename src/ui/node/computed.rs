use glam::{Vec2, Vec3};
use winit::dpi::PhysicalSize;

use crate::{prelude::Color, system::SystemsContext};

use super::{Node, Rect, Val};

#[derive(Default, Debug, Clone, Copy)]
pub struct ComputedRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
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

    pub padding: ComputedRect,
    pub margin: ComputedRect,
    pub border: ComputedRect,

    pub width: ComputedBox,
    pub height: ComputedBox,
}

// impl ComputedNode {
//     /// Returns the translation of the node by combining css properties: margin, border
//     pub fn self_translation(&self) -> Vec3 {
//         Vec3::new(
//             self.margin.left + self.border.left,
//             self.margin.top + self.border.top,
//             0.0,
//         )
//     }
//
//     pub fn children_translation(&self, parent: &ComputedNode, accumulated: &Vec3) -> Vec3 {
//         let translation = self.self_translation();
//
//         Vec3::new(
//             translation.x + parent.padding.left + accumulated.x,
//             translation.y + parent.padding.top + accumulated.y,
//             0.0,
//         )
//     }
// }

// pub struct ComputeContext<'a> {
//     pub parent_node: Option<&'a Node>,
//     pub parent_computed: Option<&'a ComputedNode>,
//     pub window_size: Vec2,
// }
//
// impl<'a> ComputeContext<'a> {
//     pub fn from_size(size: PhysicalSize<u32>) -> Self {
//         Self {
//             parent_node: None,
//             parent_computed: None,
//             window_size: Vec2::new(size.width as f32, size.height as f32),
//         }
//     }
//
//     pub fn set_parent(&mut self, parent: Option<(&'a Node, &'a ComputedNode)>) {
//         if let Some(parent) = parent {
//             self.parent_node = Some(parent.0);
//             self.parent_computed = Some(parent.1);
//         } else {
//             self.parent_node = None;
//             self.parent_computed = None;
//         }
//     }
// }

// impl Val {
//     pub fn compute(&self, parent: f32, context: &ComputeContext) -> f32 {
//         match self {
//             Val::Auto => 0.0,
//             Val::Px(val) => *val,
//             Val::Rem(val) => *val * 16.0,
//             Val::Percent(val) => parent * *val / 100.0,
//             Val::Vw(val) => context.window_size.x * *val / 100.0,
//             Val::Vh(val) => context.window_size.y * *val / 100.0,
//         }
//     }
// }
//
// impl Rect {
//     pub fn compute(&self, parent_width: f32, parent_height: f32, context: &ComputeContext) -> ComputedRect {
//         ComputedRect {
//             left: self.left.compute(parent_width, context),
//             right: self.left.compute(parent_width, context),
//             top: self.left.compute(parent_height, context),
//             bottom: self.left.compute(parent_height, context),
//         }
//     }
// }
//
// impl Node {
//     pub fn compute(&self, context: &ComputeContext, ctx: &mut SystemsContext) -> ComputedNode {
//         let parent_computed = match context.parent_computed {
//             Some(parent_computed) => parent_computed,
//             None => {
//                 let window_size = ctx.renderer.size();
//                 let window_size = (window_size.width as f32, window_size.height as f32);
//                 return self.compute_internal(window_size, context);
//             }
//         };
//
//         let size = (parent_computed.width, parent_computed.height);
//         self.compute_internal(size, context)
//     }
//
//     fn compute_internal(&self, size: (f32, f32), context: &ComputeContext) -> ComputedNode {
//         let padding = self.padding.compute(size.0, size.1, context);
//
//         ComputedNode {
//             color: self.color,
//             z_index: self.z_index + context.parent_computed.map_or(0, |parent| parent.z_index),
//
//             grid_template_columns: self.grid_template_columns.iter().map(|val| val.compute(size.0, context)).collect(),
//             grid_template_rows: self.grid_template_rows.iter().map(|val| val.compute(size.1, context)).collect(),
//
//             column_gap: match context.parent_node {
//                 Some(_) => self.column_gap.compute(size.0, context),
//                 None => 0.0,
//             },
//             row_gap: match context.parent_node {
//                 Some(_) => self.row_gap.compute(size.0, context),
//                 None => 0.0,
//             },
//
//             padding,
//             margin: self.margin.compute(size.0, size.1, context),
//             border: self.border.compute(size.0, size.1, context),
//
//             width: self.width.compute(size.0, context) + padding.left + padding.right,
//             height: self.height.compute(size.1, context) + padding.top + padding.bottom,
//         }
//     }
// }

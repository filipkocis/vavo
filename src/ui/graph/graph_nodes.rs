use crate::prelude::*;
use crate::core::graph::*;
use crate::system::CustomGraphSystem;
use crate::ui::image::render::ui_image_render_system;

use super::pipeline::{create_ui_image_pipeline_builder, create_ui_pipeline_builder};
use super::render::ui_render_system;

/// Register graph UI node
pub(crate) fn register_ui_graph(ctx: &mut SystemsContext, _: Query<()>) {
    let graph = unsafe { &mut *ctx.graph };

    let ui_image_node = ui_image_node(ctx);
    let ui_node = ui_node(ctx);

    graph.add(ui_image_node);
    graph.add(ui_node);
}

/// Create a graph UI node
fn ui_node(ctx: &mut SystemsContext) -> GraphNode {
    // Create pipeline builder
    let ui_pipeline_builder = create_ui_pipeline_builder(ctx);

    // Create graph node
    GraphNodeBuilder::new("ui")
        .set_pipeline(ui_pipeline_builder)
        .set_custom_system(CustomGraphSystem::new("ui_render_system", ui_render_system))
        .set_color_target(NodeColorTarget::Surface)
        .set_depth_target(NodeDepthTarget::Node("ui_image".to_string()))
        .run_after("ui_image")
        .build()
}

/// Create a graph UI node for images
fn ui_image_node(ctx: &mut SystemsContext) -> GraphNode {
    // Create pipeline builder
    let ui_pipeline_builder = create_ui_image_pipeline_builder(ctx);

    // Create graph node
    GraphNodeBuilder::new("ui_image")
        .set_pipeline(ui_pipeline_builder)
        .set_system(GraphSystem::new("ui_image_render_system", ui_image_render_system))
        .set_color_target(NodeColorTarget::Surface)
        .set_depth_target(NodeDepthTarget::Node("main".to_string()))
        .set_color_ops(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
        })
        .set_depth_ops(Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Store,
        }))
        .run_after("main")
        .build()
}

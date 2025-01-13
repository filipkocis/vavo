use crate::prelude::*;
use crate::core::graph::*;
use crate::system::CustomGraphSystem;

use super::pipeline::create_ui_pipeline_builder;
use super::render::ui_render_system;

/// Register graph UI node
pub(super) fn register_ui_graph(ctx: &mut SystemsContext, _: Query<()>) {
    let graph = unsafe { &mut *ctx.graph };

    let ui_node = ui_node(ctx);
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
        .set_depth_target(NodeDepthTarget::Node("main".to_string()))
        .add_dependency("main")
        .add_dependency("shadow")
        .build()
}

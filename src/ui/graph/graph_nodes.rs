use crate::core::graph::*;
use crate::prelude::*;
use crate::renderer::newtype::{RenderDevice, RenderSurfaceConfiguration};
use crate::system::CustomGraphSystem;
use crate::ui::image::render::ui_image_render_system;

use super::pipeline::{create_ui_image_pipeline_builder, create_ui_pipeline_builder};
use super::render::ui_render_system;

/// Register graph UI node
pub(crate) fn register_ui_graph(
    graph: &mut RenderGraph,
    device: Res<RenderDevice>,
    surface_config: Res<RenderSurfaceConfiguration>,
    mut shader_loader: ResMut<ShaderLoader>,
) {
    let ui_image_node = ui_image_node(&device, &surface_config, &mut shader_loader);
    let ui_node = ui_node(&device, &surface_config, &mut shader_loader);

    graph.add(ui_image_node);
    graph.add(ui_node);
}

/// Create a graph UI node
fn ui_node(
    device: &RenderDevice,
    surface_config: &RenderSurfaceConfiguration,
    shader_loader: &mut ShaderLoader,
) -> GraphNode {
    // Create pipeline builder
    let ui_pipeline_builder = create_ui_pipeline_builder(device, surface_config, shader_loader);

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
fn ui_image_node(
    device: &RenderDevice,
    surface_config: &RenderSurfaceConfiguration,
    shader_loader: &mut ShaderLoader,
) -> GraphNode {
    // Create pipeline builder
    let ui_pipeline_builder =
        create_ui_image_pipeline_builder(device, surface_config, shader_loader);

    // Create graph node
    GraphNodeBuilder::new("ui_image")
        .set_pipeline(ui_pipeline_builder)
        .set_system(GraphSystem::new(
            "ui_image_render_system",
            ui_image_render_system,
        ))
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

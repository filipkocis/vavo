mod pipeline;
mod render;
mod storage;
mod compute;
mod update;
mod build_temp;

use compute::compute_nodes_and_transforms;
use update::{update_glyphon_viewport, update_ui_mesh_and_transforms};
use glyphon::{FontSystem, SwashCache, Cache, Viewport, TextAtlas, TextRenderer};
use pipeline::create_ui_pipeline_builder;
use render::{initialize_ui_nodes, ui_render_system};
pub use storage::UiTransformStorage;

use crate::prelude::*;
use crate::core::graph::*;
use crate::render_assets::RenderAssets;
use super::text::TextBuffer;

/// Create a graph UI node
fn ui_node(ctx: &mut SystemsContext) -> GraphNode {
    // Create pipeline builder
    let ui_pipeline_builder = create_ui_pipeline_builder(ctx);

    // Create graph node
    GraphNodeBuilder::new("ui")
        .set_pipeline(ui_pipeline_builder)
        .set_system(GraphSystem::new("ui_render_system", ui_render_system))
        .set_color_target(NodeColorTarget::Surface)
        .set_depth_target(NodeDepthTarget::Node("main".to_string()))
        // .set_depth_target(NodeDepthTarget::None)
        .add_dependency("main")
        .add_dependency("shadow")
        .set_depth_ops(Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Store,
        }))
        .set_color_ops(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
        }) 
        .build()
}

/// Inset necessary UI text resources to app
fn insert_ui_text_resources(ctx: &mut SystemsContext, _: Query<()>) {
    let device = ctx.renderer.device();
    let queue = ctx.renderer.queue();
    let swapchain_format = ctx.renderer.config().format;

    let font_system = FontSystem::new();
    let swash_cache = SwashCache::new();
    let cache = Cache::new(&device);
    let viewport = Viewport::new(&device, &cache);
    let mut atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
    let text_renderer = TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(), 
    }));

    ctx.resources.insert(font_system);
    ctx.resources.insert(swash_cache);
    ctx.resources.insert(viewport);
    ctx.resources.insert(atlas);
    ctx.resources.insert(text_renderer);
    ctx.resources.insert(RenderAssets::<TextBuffer>::new());
}

/// Inset necessary UI resources to app
fn insert_ui_resources(ctx: &mut SystemsContext, _: Query<()>) {
    let node_transform_storage = UiTransformStorage::new(1, 32, ctx, wgpu::ShaderStages::VERTEX);
    let ui_mesh = UiMesh::new();

    ctx.resources.insert(node_transform_storage);
    ctx.resources.insert(ui_mesh);
}

/// Register graph UI node
fn register_ui_graph(ctx: &mut SystemsContext, _: Query<()>) {
    let graph = unsafe { &mut *ctx.graph };

    let ui_node = ui_node(ctx);
    graph.add(ui_node);
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(insert_ui_resources) 
            .add_startup_system(insert_ui_text_resources) 
            .add_startup_system(register_ui_graph)
            .register_system(initialize_ui_nodes, SystemStage::PreUpdate)
            .register_system(compute_nodes_and_transforms, SystemStage::PostUpdate)
            .register_system(update_glyphon_viewport, SystemStage::PreRender)
            .register_system(update_ui_mesh_and_transforms, SystemStage::PreRender);
    }
}

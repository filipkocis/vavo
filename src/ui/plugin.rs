use glyphon::{FontSystem, SwashCache, Cache, Viewport, TextAtlas, TextRenderer};

use super::{graph::{
    compute::compute_nodes_and_transforms, graph_nodes::register_ui_graph, storage::UiTransformStorage, update::{update_glyphon_viewport, update_ui_mesh_and_transforms}
}, interactivity::{ui_interaction_update, Button}};

use crate::{prelude::*, ui::interactivity::Interaction};
use crate::render_assets::RenderAssets;
use super::text::TextBuffer;

/// System to initialize new UI nodes, it adds Transform and ComputedNode components
pub fn initialize_ui_nodes(
    ctx: &mut SystemsContext,
    mut query: Query<&EntityId, (With<Node>, Without<Transform>, Without<ComputedNode>)>,
) {
    for id in query.iter_mut() {
        ctx.commands.entity(*id)
            .insert(Transform::default())
            .insert(ComputedNode::default());

        println!("Initialized ui node: {:?}", id);
    }
}

/// System to initialize new button UI nodes, adds Interaction component
pub fn initialize_button_ui_nodes(
    ctx: &mut SystemsContext,
    mut query: Query<&EntityId, (With<Node>, With<Button>, Without<Interaction>)>,
) {
    for id in query.iter_mut() {
        ctx.commands.entity(*id)
            .insert(Interaction::default());

        println!("Initialized button ui node: {:?}", id);
    }
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
    let ui_mesh_transparent = UiMeshTransparent::new();

    ctx.resources.insert(node_transform_storage);
    ctx.resources.insert(ui_mesh);
    ctx.resources.insert(ui_mesh_transparent);
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(insert_ui_resources) 
            .add_startup_system(insert_ui_text_resources) 
            .add_startup_system(register_ui_graph)
            .register_system(ui_interaction_update, SystemStage::First)
            .register_system(initialize_ui_nodes, SystemStage::PreUpdate)
            .register_system(initialize_button_ui_nodes, SystemStage::PreUpdate)
            .register_system(compute_nodes_and_transforms, SystemStage::PostUpdate)
            .register_system(update_glyphon_viewport, SystemStage::PreRender)
            .register_system(update_ui_mesh_and_transforms, SystemStage::PreRender);
    }
}

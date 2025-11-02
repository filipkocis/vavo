use glyphon::{Cache, FontSystem, SwashCache, TextAtlas, TextRenderer, Viewport};

use super::{
    graph::{
        compute::compute_nodes_and_transforms,
        graph_nodes::register_ui_graph,
        storage::UiTransformStorage,
        update::{update_glyphon_viewport, update_ui_mesh_and_transforms},
    },
    interactivity::{Button, ui_interaction_update},
    mesh::{UiMesh, UiMeshImages, UiMeshTransparent},
};

use super::text::TextBuffer;
use crate::{
    prelude::*,
    renderer::newtype::{RenderQueue, RenderSurfaceConfiguration},
    ui::prelude::*,
};
use crate::{render_assets::RenderAssets, renderer::newtype::RenderDevice};

/// System to initialize new UI nodes, it adds Transform and ComputedNode components
pub fn initialize_ui_nodes(
    mut commands: Commands,
    mut query: Query<EntityId, (With<Node>, Without<Transform>, Without<ComputedNode>)>,
) {
    for id in query.iter_mut() {
        commands
            .entity(id)
            .insert(Transform::default())
            .insert(ComputedNode::default());
    }
}

/// System to initialize new button UI nodes, adds Interaction component
pub fn initialize_button_ui_nodes(
    mut commands: Commands,
    mut query: Query<EntityId, (With<Node>, With<Button>, Without<Interaction>)>,
) {
    for id in query.iter_mut() {
        commands.entity(id).insert(Interaction::default());
    }
}

/// Inset necessary UI text resources to app
fn insert_ui_text_resources(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    surface_config: Res<RenderSurfaceConfiguration>,
) {
    let swapchain_format = surface_config.format;

    let font_system = FontSystem::new();
    let swash_cache = SwashCache::new();
    let cache = Cache::new(&device);
    let viewport = Viewport::new(&device, &cache);
    let mut atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
    let text_renderer = TextRenderer::new(
        &mut atlas,
        &device,
        wgpu::MultisampleState::default(),
        Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
    );

    commands
        .insert_resource(font_system)
        .insert_resource(swash_cache)
        .insert_resource(viewport)
        .insert_resource(atlas)
        .insert_resource(text_renderer)
        .insert_resource(RenderAssets::<TextBuffer>::new());
}

/// Inset necessary UI resources to app
fn insert_ui_resources(mut commands: Commands, device: Res<RenderDevice>) {
    let node_transform_storage =
        UiTransformStorage::new(1, 32, &device, wgpu::ShaderStages::VERTEX);
    let ui_mesh = UiMesh::new();
    let ui_mesh_transparent = UiMeshTransparent::new();
    let ui_mesh_images = UiMeshImages::new();

    commands
        .insert_resource(node_transform_storage)
        .insert_resource(ui_mesh)
        .insert_resource(ui_mesh_transparent)
        .insert_resource(ui_mesh_images);
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(insert_ui_resources)
            .add_startup_system(insert_ui_text_resources)
            .add_startup_system(register_ui_graph)
            .register_system(ui_interaction_update, phase::First)
            .register_system(initialize_ui_nodes, phase::PreUpdate)
            .register_system(initialize_button_ui_nodes, phase::PreUpdate)
            .register_system(compute_nodes_and_transforms, phase::PostUpdate)
            .register_system(update_glyphon_viewport, phase::PreRender)
            .register_system(update_ui_mesh_and_transforms, phase::PreRender);
    }
}

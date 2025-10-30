use crate::{
    prelude::*,
    render_assets::TransformStorage,
    renderer::newtype::{RenderDevice, RenderSurfaceConfiguration, RenderWindow},
};

use super::{rendering::standard_main_node, shadows::standard_shadow_node};

/// Internal system to add necessary resources for standard rendering
pub fn add_render_resources(mut commands: Commands, device: Res<RenderDevice>) {
    let storage = TransformStorage::new(100, 64, &device, wgpu::ShaderStages::VERTEX);
    commands.insert_resource(storage);
}

/// Startup system to register standard render graph
pub fn register_standard_graph(
    world: &mut World,
    device: Res<RenderDevice>,
    mut shader_loader: ResMut<ShaderLoader>,
    surface_config: Res<RenderSurfaceConfiguration>,
    window: Res<RenderWindow>,

    ctx: &mut SystemsContext,
) {
    let graph = unsafe { &mut *ctx.graph };

    let main_node = standard_main_node(&device, &mut shader_loader, &surface_config, &window);
    graph.add(main_node);

    let shadow_node = standard_shadow_node(&device, &mut shader_loader, world);
    graph.add(shadow_node);
}

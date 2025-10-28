use crate::{prelude::*, render_assets::TransformStorage, renderer::newtype::RenderDevice};

use super::{rendering::standard_main_node, shadows::standard_shadow_node};

/// Internal system to add necessary resources for standard rendering
pub fn add_render_resources(mut commands: Commands, device: Res<RenderDevice>) {
    let storage = TransformStorage::new(100, 64, &device, wgpu::ShaderStages::VERTEX);
    commands.insert_resource(storage);
}

/// Startup system to register standard render graph
pub fn register_standard_graph(ctx: &mut SystemsContext, _: Query<()>) {
    let graph = unsafe { &mut *ctx.graph };

    let main_node = standard_main_node(ctx);
    graph.add(main_node);

    let shadow_node = standard_shadow_node(ctx);
    graph.add(shadow_node);
}

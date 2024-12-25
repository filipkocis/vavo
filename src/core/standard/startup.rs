use crate::{prelude::*, render_assets::TransformStorage};

use super::{rendering::standard_main_node, shadows::standard_shadow_node};

/// Internal system to add necessary resources for standard rendering
pub fn add_resources(ctx: &mut SystemsContext, _: Query<()>) {
    let storage = TransformStorage::new(100, 64, ctx, wgpu::ShaderStages::VERTEX);
    ctx.resources.insert(storage);
}

/// Startup system to register standard render graph
pub fn register_standard_graph(ctx: &mut SystemsContext, _: Query<()>) {
    let graph = unsafe { &mut *ctx.graph }; 

    let main_node = standard_main_node(ctx);   
    graph.add(main_node);

    let shadow_node = standard_shadow_node(ctx);
    graph.add(shadow_node);
}

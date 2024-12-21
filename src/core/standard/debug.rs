use pipeline::PipelineBuilder;

use crate::{core::graph::*, prelude::*, render_assets::*};

use super::atlas::ShadowMapAtlas;

pub fn debug_shadow_map_node(ctx: &mut SystemsContext) -> GraphNode {
    // Create pipeline builder
    let debug_shadow_pipeline_builder = create_debug_shadow_pipeline_builder(
        ctx.renderer.device(), 
        ctx.renderer.config().format
    );

    // Create graph node
    GraphNodeBuilder::new("debug_shadow")
        .set_pipeline(debug_shadow_pipeline_builder)
        .set_system(GraphSystem::new("debug_shadow_render_system", debug_shadow_render_system))
        .set_color_target(NodeColorTarget::Surface)
        .set_depth_target(NodeDepthTarget::None)
        .add_dependency("shadow")
        .build()
}

fn debug_shadow_render_system(
    graph_ctx: RenderGraphContext, 
    ctx: &mut SystemsContext, 
    _: Query<()>
) {
    let shadow_map_atlas = ctx.resources.get::<ShadowMapAtlas>().expect("ShadowMapAtlas resource not found");

    let mut bind_groups = ctx.resources.get_mut::<RenderAssets<BindGroup>>().unwrap();
    let atlas_bind_group = bind_groups.get_by_resource(&shadow_map_atlas, ctx, false);

    let render_pass = graph_ctx.pass;
    render_pass.set_bind_group(0, &*atlas_bind_group, &[]);

    // draw screen triangle
    render_pass.draw(0..3, 0..1);
}

fn create_debug_shadow_pipeline_builder(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> PipelineBuilder {
    // Shadow map bind group layout for texture and sampler
    let shadow_map_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("shadow_map_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false,
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None
            }
        ]
    });

    // Create builder
    Pipeline::build("debug_shadow_pipeline")
        .set_bind_group_layouts(vec![shadow_map_layout])
        .set_vertex_shader(include_str!("../../shaders/debug-shadow.wgsl"), "vs_main")
        .set_fragment_shader(include_str!("../../shaders/debug-shadow.wgsl"), "fs_main")
        .set_color_format(texture_format)
}

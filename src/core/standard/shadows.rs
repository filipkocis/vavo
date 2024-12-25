use pipeline::PipelineBuilder;

use crate::{core::{graph::*, lighting::LightAndShadowManager}, prelude::*, render_assets::*, system::CustomGraphSystem};

use super::{grouped::GroupedInstances, light_data::PreparedLightData};

pub fn standard_shadow_node(ctx: &mut SystemsContext) -> GraphNode {
    // Create pipeline builder
    let shadow_pipeline_builder = create_shadow_pipeline_builder(ctx.renderer.device());

    // Create light and shadow manager
    let manager = LightAndShadowManager::new(ctx);
    ctx.resources.insert(manager);
    
    // Create graph node
    GraphNodeBuilder::new("shadow")
        .set_pipeline(shadow_pipeline_builder)
        .set_custom_system(CustomGraphSystem::new("shadow_render_system", shadow_render_system))
        // TODO: fix loop dependency
        // .add_dependency("main")
        .build()
}

fn shadow_render_system(
    graph_ctx: CustomRenderGraphContext, 
    ctx: &mut SystemsContext, 
    _: Query<()>
) {
    // Render assets
    let mut buffers = ctx.resources.get_mut::<RenderAssets<Buffer>>().unwrap();

    // Resources
    let light_manager = ctx.resources.get::<LightAndShadowManager>().expect("LightAndShadowManager resource not found");
    let transforms_storage = ctx.resources.get::<TransformStorage>().expect("TransformStorag resource not found");  

    // Resources from preparation system
    let grouped = ctx.resources.get::<GroupedInstances>().expect("GroupedInstances resource not found");  
    let light_data = ctx.resources.get::<PreparedLightData>().expect("PreparedLightData resource not found");

    // Get node's pipeline
    let pipeline = unsafe { &*graph_ctx.node }.data.pipeline.as_ref().expect("Pipeline should have been generated by now").render_pipeline();

    // Instanced per light
    for i in 0..light_data.lights.len() {
        let light = &light_data.lights[i];

        if light.is_ambient() || !light.is_shadowed() || !light.is_visible() {
            continue;
        }

        per_light_render_pass(i as u32, &light, &grouped, &transforms_storage, &light_manager, pipeline, &mut buffers, ctx);
    }
}

fn per_light_render_pass(
    light_index: u32,
    light: &Light,
    grouped: &GroupedInstances, 
    transforms_storage: &TransformStorage,
    light_manager: &LightAndShadowManager,
    pipeline: &wgpu::RenderPipeline,
    buffers: &mut RenderAssets<Buffer>, 
    ctx: &mut SystemsContext,
) {
    // Create render pass with the correct layer in the shadow map
    let encoder = ctx.renderer.encoder().inner;
    let mut render_pass = unsafe { &mut *encoder }.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("shadow render pass"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &light_manager.create_view(light),
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    // Set pipeline
    render_pass.set_pipeline(pipeline);

    // Set bind groups for transforms and lights
    render_pass.set_bind_group(0, transforms_storage.bind_group(), &[]);
    render_pass.set_bind_group(1, light_manager.storage.bind_group(), &[]);

    // Set light index
    render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::bytes_of(&light_index));
    
    // Instanced draw loop
    let materials = ctx.resources.get::<Assets<Material>>().unwrap();
    let mut last_mesh = None;
    for group in &grouped.groups {
        let material = &group.material;
        let mesh = &group.mesh;
        let instance_count = group.instance_count;
        let instance_offset = group.instance_offset;

        // check unlit material
        if let Some(material) = materials.get(material) {
            if material.unlit {
                continue;
            }
        }

        // set vertex buffer with mesh
        let mesh_buffer = buffers.get_by_handle(mesh, ctx); 
        if last_mesh != Some(mesh) {
            render_pass.set_vertex_buffer(0, mesh_buffer.vertex.as_ref()
                .expect("mesh should have vertex buffer").slice(..));
        }

        // draw
        let instance_range = instance_offset..(instance_offset + instance_count);
        if let Some(index_buffer) = &mesh_buffer.index {
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh_buffer.num_indices, 0, instance_range);
        } else {
            render_pass.draw(0..mesh_buffer.num_vertices, instance_range);
        }

        last_mesh = Some(mesh);
    }
}

fn create_shadow_pipeline_builder(device: &wgpu::Device) -> PipelineBuilder {
    // Transform bind group layout for storage buffer
    let transforms_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("transforms_bind_group_layout"), 
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false, 
                    min_binding_size: None 
                },
                count: None
            }
        ]
    });

    // Light bind group layout for storage buffer
    let lights_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("lights_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            },
        ]
    });

    // Create builder
    Pipeline::build("shadows_pipeline")
        .set_bind_group_layouts(vec![transforms_layout, lights_layout])
        .set_vertex_buffer_layouts(vec![Mesh::vertex_descriptor()])
        .set_vertex_shader(include_str!("../../shaders/shadow.wgsl"), "vs_main")
        .set_depth_format(wgpu::TextureFormat::Depth32Float)
        .set_push_constant_ranges(vec![wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::VERTEX,
            range: 0..4,
        }])
}

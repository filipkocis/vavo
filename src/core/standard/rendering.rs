use pipeline::PipelineBuilder;
use wgpu::TextureFormat;

use crate::{core::{graph::*, lighting::LightAndShadowManager}, prelude::*, render_assets::*};

use super::grouped::GroupedInstances;

/// Creates a node for standard main render pass
pub fn standard_main_node(ctx: &mut SystemsContext) -> GraphNode {
    // Create pipeline builder
    let main_pipeline_builder = create_main_pipeline_builder(
        ctx.renderer.device(), 
        ctx.renderer.config().format
    );

    // Create depth image
    let size = ctx.renderer.window().inner_size();
    let mut depth_image = Image::new_with_defaults(vec![], wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    });

    // Change defaults for depth image
    depth_image.texture_descriptor.as_mut().unwrap().format = wgpu::TextureFormat::Depth32Float;
    depth_image.texture_descriptor.as_mut().unwrap().view_formats = &[]; 
    depth_image.texture_descriptor.as_mut().unwrap().usage = wgpu::TextureUsages::RENDER_ATTACHMENT; 
    depth_image.view_descriptor.as_mut().unwrap().format = Some(wgpu::TextureFormat::Depth32Float);

    GraphNodeBuilder::new("main")
        .set_pipeline(main_pipeline_builder)
        .set_system(GraphSystem::new("main_render_system", main_render_system))
        .set_color_target(NodeColorTarget::Surface)
        .set_depth_target(NodeDepthTarget::Owned(depth_image))
        .add_dependency("shadow")
        .build()
}

fn main_render_system(
    graph_ctx: RenderGraphContext, 
    ctx: &mut SystemsContext, 
    mut query: Query<()>
) {
    // Render assets
    let mut buffers = ctx.resources.get_mut::<RenderAssets<Buffer>>().unwrap();
    let mut bind_groups = ctx.resources.get_mut::<RenderAssets<BindGroup>>().unwrap();

    // Resources
    let manager = ctx.resources.get::<LightAndShadowManager>().expect("LightAndShadowManager not found");
    let grouped = ctx.resources.get::<GroupedInstances>().expect("GroupedInstances resource not found");
    let transforms_storage = ctx.resources.get::<TransformStorage>().expect("TransformStorage resource not found");

    // Camera
    let mut camera_query = query.cast::<(&EntityId, &Camera), (With<Transform>, With<Projection>, With<Camera3D>)>(); 
    let active_camera = camera_query.iter_mut().into_iter().filter(|(_, c)| c.active).take(1).next();
    let camera_bind_group;
    if let Some((id, camera)) = active_camera {
        camera_bind_group = bind_groups.get_by_entity(id, camera, ctx);
    } else {
        return;
    }

    let render_pass = graph_ctx.pass;

    // Set light count push constant
    render_pass.set_push_constants(wgpu::ShaderStages::FRAGMENT, 0, bytemuck::cast_slice(&[
        manager.storage.count() as u32
    ]));

    // TODO: currently we have to regen every time, because manager views got updated
    let manager_bind_group = bind_groups.get_by_resource(&manager, ctx, true);

    // Set bind groups
    render_pass.set_bind_group(1, transforms_storage.bind_group(), &[]);
    render_pass.set_bind_group(2, &*camera_bind_group, &[]);
    render_pass.set_bind_group(3, &*manager_bind_group, &[]);

    // Instanced draw loop
    let mut last_material = None;
    let mut last_mesh = None;
    // for (material, mesh, instance_count, instance_offset) in grouped {
    for group in &grouped.groups {
        let material = &group.material;
        let mesh = &group.mesh;
        let instance_count = group.instance_count;
        let instance_offset = group.instance_offset;

        // bind material
        if last_material != Some(material) {
            let material_bind_group = bind_groups.get_by_handle(material, ctx); 
            render_pass.set_bind_group(0, &*material_bind_group, &[]);
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

        last_material = Some(material);
        last_mesh = Some(mesh);
    }
}

// TODO: add a better way to generate/get bind group layouts
fn create_main_pipeline_builder(device: &wgpu::Device, color_format: TextureFormat) -> PipelineBuilder {
    // Material bind group layout for texture and uniform buffer
    let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("material_bind_group_layout"), 
        entries: &[
            // base texture
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false 
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // normal map
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false 
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // uniform buffer
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None 
                },
                count: None
            }
        ]
    });

    // Transform bind group layout for storage buffer
    let transform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("transform_bind_group_layout"), 
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

    // Camera bind group layout for uniform buffer
    let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("camera_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }
        ]
    });

    // Light and shadow manager
    let manager_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("light_and_shadow_manager_layout"),
        entries: &[
            // lights storage buffer
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
            // directional shadow map
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Depth, 
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false
                },
                count: None
            },
            // point shadow map
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Depth, 
                    view_dimension: wgpu::TextureViewDimension::CubeArray,
                    multisampled: false
                },
                count: None
            },
            // spot shadow map
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Depth, 
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false
                },
                count: None
            },
            // shadow map sampler
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None
            }
        ]
    });

    // Create builder
    Pipeline::build("main_pipeline")
        .set_bind_group_layouts(vec![material_layout, transform_layout, camera_layout, manager_layout])
        .set_vertex_buffer_layouts(vec![Mesh::vertex_descriptor()])
        .set_vertex_shader(include_str!("../../shaders/shader.wgsl"), "vs_main")
        .set_fragment_shader(include_str!("../../shaders/shader.wgsl"), "fs_main")
        .set_color_format(color_format)
        .set_depth_format(wgpu::TextureFormat::Depth32Float)
        .set_push_constant_ranges(vec![wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::FRAGMENT,
            range: 0..4,
        }])
}

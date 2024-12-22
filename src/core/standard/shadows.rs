use glam::{Mat4, Vec2, Vec4Swizzles};
use pipeline::PipelineBuilder;

use crate::{core::graph::*, prelude::*, render_assets::*};

use super::atlas::ShadowMapAtlas;

const ATLAS_TILE_SIZE: u32 = 1024;
const ATLAS_ROWS: u32 = 5;
const ATLAS_COLS: u32 = 5;

pub fn standard_shadow_node(ctx: &mut SystemsContext) -> GraphNode {
    // Create pipeline builder
    let shadow_pipeline_builder = create_shadow_pipeline_builder(ctx.renderer.device());

    // Create shadow map atlas
    let shadow_map_atlas = ShadowMapAtlas::new(
        (ATLAS_TILE_SIZE, ATLAS_TILE_SIZE), 
        ATLAS_ROWS, 
        ATLAS_COLS, 
        &mut ctx.resources.get_mut::<Assets<Image>>().unwrap()
    );

    let atlas_image = shadow_map_atlas.image.clone();
    ctx.resources.insert(shadow_map_atlas);
    
    // Create graph node
    GraphNodeBuilder::new("shadow")
        .set_pipeline(shadow_pipeline_builder)
        .set_system(GraphSystem::new("shadow_render_system", shadow_render_system))
        .set_color_target(NodeColorTarget::None)
        .set_depth_target(NodeDepthTarget::Image(atlas_image))
        .set_depth_ops(Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Store,
        }))
        .add_dependency("main")
        .build()
}

fn shadow_render_system<'a>(
    graph_ctx: RenderGraphContext, 
    ctx: &mut SystemsContext, 
    mut query: Query<'a, (&'a Handle<Material>, &'a Handle<Mesh>, &'a GlobalTransform)>
) {
    // Render assets
    let mut buffers = ctx.resources.get_mut::<RenderAssets<Buffer>>().unwrap();

    // Extract camera position
    let mut camera_query = query.cast::<(&GlobalTransform, &Camera), (With<Projection>, With<Camera3D>)>(); 
    let active_camera = camera_query.iter_mut().into_iter().filter(|(_, c)| c.active).take(1).next();
    let camera_position = match active_camera.map(|(t, _)| t.matrix.w_axis.xyz()) {
        Some(p) => p,
        None => return
    };

    // Prepare light data
    let mut light_data = Vec::new();
    let mut directional_query = query.cast::<(&GlobalTransform, &DirectionalLight), ()>();
    for (global_transform, light) in directional_query.iter_mut() {
        if !light.shadow {
            continue;
        }

        let view_projection_matrix = light.view_projection_matrix(50.0, 0.1, 100.0, camera_position, global_transform.matrix);

        light_data.push(light.as_light(view_projection_matrix))
    }
    let mut spot_query = query.cast::<(&GlobalTransform, &SpotLight), ()>();
    for (global_transform, light) in spot_query.iter_mut() {
        if !light.shadow {
            continue;
        }

        let view_projection_matrix = light.view_projection_matrix(1.0, 0.1, global_transform.matrix);

        light_data.push(light.as_light(view_projection_matrix))
    }
    if let Some(light) = ctx.resources.get::<AmbientLight>() {
        light_data.push(light.as_light(Mat4::IDENTITY))
    };

    // Prepare sorted storage
    let materials = ctx.resources.get::<Assets<Material>>().unwrap();
    let mut transforms = Vec::new();
    let mut sorted = Vec::<(&Handle<Mesh>, &GlobalTransform)>::new();
    for (mat, mesh, global_transform) in query.iter_mut() {
        if materials.get(mat).expect("Material not found").unlit {
            continue;
        }

        sorted.push((mesh, global_transform));
    }

    // Sort by mesh
    sorted.sort_by(|a, b| a.0.id().cmp(&b.0.id()));

    // Group by mesh
    let last_index = sorted.len() - 1;
    let mut last_entry = None;
    let mut instance_count = 0;
    let mut instance_offset = 0;
    let mut grouped = Vec::<(&Handle<Mesh>, u32, u32)>::new();
    for (i, (mesh, global_transform)) in sorted.into_iter().enumerate() {
        if let Some((last_mesh, last_instance_count)) = last_entry {
            if last_mesh == mesh {
                instance_count += 1;
            } else {
                grouped.push((last_mesh, last_instance_count, instance_offset));
                instance_offset += last_instance_count;
                instance_count = 1;
            }
        } else {
            instance_count = 1;
        }

        if i == last_index {
            grouped.push((mesh, instance_count, instance_offset));
        }

        last_entry = Some((mesh, instance_count));
        transforms.push(global_transform.as_matrix().to_cols_array_2d());
    }

    let render_pass = graph_ctx.pass;

    // Set transforms storage
    let mut transforms_storage = ctx.resources.get_mut::<TransformStorage>().unwrap();
    transforms_storage.update(&transforms, ctx);
    render_pass.set_bind_group(0, transforms_storage.bind_group(), &[]);
    
    // Set lights storage
    let mut lights_storage = ctx.resources.get_mut::<LightStorage>().unwrap();
    lights_storage.update(&light_data, ctx);
    render_pass.set_bind_group(1, lights_storage.bind_group(), &[]);

    // Instanced per light
    for i in 0..light_data.len() as u32 {
        // Set light index
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::bytes_of(&i));

        // Set atlas tile viewport
        let tile = Vec2::new((i % ATLAS_COLS) as f32, (i / ATLAS_ROWS) as f32);
        let tile_size = Vec2::new(ATLAS_TILE_SIZE as f32, ATLAS_TILE_SIZE as f32);
        render_pass.set_viewport(
            tile.x * tile_size.x,
            tile.y * tile_size.y,
            tile_size.x,
            tile_size.y,
            0.0,
            1.0,
        );
        
        // Instanced draw loop
        let mut last_mesh = None;
        for (mesh, instance_count, instance_offset) in &grouped {
            // set vertex buffer with mesh
            let mesh_buffer = buffers.get_by_handle(mesh, ctx); 
            if last_mesh != Some(*mesh) {
                render_pass.set_vertex_buffer(0, mesh_buffer.vertex.as_ref()
                    .expect("mesh should have vertex buffer").slice(..));
            }

            // draw
            let instance_range = *instance_offset..(*instance_offset + *instance_count);
            if let Some(index_buffer) = &mesh_buffer.index {
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh_buffer.num_indices, 0, instance_range);
            } else {
                render_pass.draw(0..mesh_buffer.num_vertices, instance_range);
            }

            last_mesh = Some(*mesh);
        }
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
                visibility: wgpu::ShaderStages::VERTEX,
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

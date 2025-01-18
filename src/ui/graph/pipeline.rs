use crate::assets::ShaderLoader;
use crate::prelude::*;
use crate::render_assets::{pipeline::PipelineBuilder, Pipeline};
use crate::ui::mesh::UiMesh;

pub fn create_ui_pipeline_builder(ctx: &mut SystemsContext) -> PipelineBuilder {
    let device = ctx.renderer.device();
    let color_format = ctx.renderer.config().format;

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

    // Load shader modules
    ctx.resources.get_mut::<ShaderLoader>().expect("ShaderLoader resource not found")
        .load("ui", include_str!("../../shaders/ui.wgsl"), device);

    // Create pipeline builder
    Pipeline::build("ui")
        .set_bind_group_layouts(vec![transform_layout, camera_layout])
        .set_vertex_buffer_layouts(vec![UiMesh::vertex_descriptor()])
        .set_vertex_shader("ui", "vs_main")
        .set_fragment_shader("ui", "fs_main")
        .set_color_format(color_format)
        .set_depth_format(wgpu::TextureFormat::Depth32Float)
        .set_push_constant_ranges(vec![
            wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..4 * 2,
            }
        ])
}

pub fn create_ui_image_pipeline_builder(ctx: &mut SystemsContext) -> PipelineBuilder {
    let mut pipeline_builder = create_ui_pipeline_builder(ctx);

    // Image bind group layout for uniform buffer
    let image_layout = ctx.renderer.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("image_bind_group_layout"),
        entries: &[
            // texture
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
            // sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // uniform
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform,
                    min_binding_size: None,
                    has_dynamic_offset: false,
                },
                count: None,
            }
        ]
    });

    let bind_group_layouts = pipeline_builder.bind_group_layouts.as_mut()
        .expect("ui bind group layouts should not be empty");

    bind_group_layouts.push(image_layout);

    pipeline_builder
        .set_label("ui_image")
        .set_vertex_shader("ui", "vs_image")
        .set_fragment_shader("ui", "fs_image")
}

use crate::{assets::ShaderLoader, system::SystemsContext};

use super::RenderHandle;

#[derive(crate::macros::RenderAsset)]
pub struct Pipeline {
    inner: wgpu::RenderPipeline,
}

pub struct StandardPipeline {
    pub handle: RenderHandle<Pipeline>,
}

impl StandardPipeline {
    pub fn new(handle: RenderHandle<Pipeline>) -> Self {
        Self {
            handle,
        }
    }
}

impl Pipeline {
    /// Creates a new instance of PipelineBuilder
    pub fn build(label: &str) -> PipelineBuilder {
        PipelineBuilder::new(label)
    }

    /// Return the inner wgpu::RenderPipeline
    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.inner
    }
}

pub struct PipelineBuilder {
    pub label: String,
    pub bind_group_layouts: Option<Vec<wgpu::BindGroupLayout>>,
    pub vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    pub vertex_shader: Option<(String, String)>,
    pub fragment_shader: Option<(String, String)>,
    pub color_format: Option<wgpu::TextureFormat>,
    pub depth_format: Option<wgpu::TextureFormat>,
    pub topology: Option<wgpu::PrimitiveTopology>,
    pub push_constant_ranges: Vec<wgpu::PushConstantRange>,
}

impl PipelineBuilder {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            bind_group_layouts: None,
            vertex_buffer_layouts: Vec::new(),
            vertex_shader: None,
            fragment_shader: None,
            color_format: None,
            depth_format: None,
            topology: None,
            push_constant_ranges: Vec::new(),
        }
    }

    /// Set new label, useful when creating a pipeline from a fn created 'base' pipeline
    pub fn set_label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    /// Set bind group layouts for pipeline layout
    ///
    /// # Note
    /// If this is not set, the pipeline layout will be None
    pub fn set_bind_group_layouts(mut self, layouts: Vec<wgpu::BindGroupLayout>) -> Self {
        self.bind_group_layouts = Some(layouts);
        self
    }

    /// Set vertex buffer layouts for vertex state
    ///
    /// # Note
    /// Default is an empty slice
    pub fn set_vertex_buffer_layouts(mut self, layouts: Vec<wgpu::VertexBufferLayout<'static>>) -> Self {
        self.vertex_buffer_layouts = layouts;
        self
    }

    /// Set a vertex shader
    ///
    /// # Note
    /// Label is the name of a loaded shader in ShaderLoader.
    /// This is required.
    pub fn set_vertex_shader(mut self, label: &str, entry_point: &str) -> Self {
        self.vertex_shader = Some((label.to_string(), entry_point.to_string()));
        self
    }

    /// Set a fragment shader
    ///
    /// # Note
    /// Label is the name of a loaded shader in ShaderLoader.
    /// If not set, the fragment state will be None
    pub fn set_fragment_shader(mut self, label: &str, entry_point: &str) -> Self {
        self.fragment_shader = Some((label.to_string(), entry_point.to_string()));
        self
    }

    /// Set the texture format color target in fragment state
    ///
    /// # Note
    /// For this to be used, you must set a fragment shader
    pub fn set_color_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.color_format = Some(format);
        self
    }

    /// Set texture format for depth stencil
    ///
    /// # Note
    /// If not set, the depth stencil state will be None
    pub fn set_depth_format(mut self, depth_format: wgpu::TextureFormat) -> Self {
        self.depth_format = Some(depth_format);
        self
    }
    
    /// Set primitive topology for the pipeline
    ///
    /// # Note
    /// Default is TriangleList
    pub fn set_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.topology = Some(topology);
        self
    }

    /// Set push constant ranges for pipeline layout
    pub fn set_push_constant_ranges(mut self, ranges: Vec<wgpu::PushConstantRange>) -> Self {
        self.push_constant_ranges = ranges;
        self
    }

    fn load_shader<'a>(&self, label_entry: &Option<(String, String)>, shader_loader: &'a ShaderLoader) -> (&'a wgpu::ShaderModule, String) {
        self.load_shader_maybe(label_entry, shader_loader)
            .expect(&format!("{} shader for {} not set", label_entry.as_ref().unwrap().0, self.label))
    }

    fn load_shader_maybe<'a>(&self, label_entry: &Option<(String, String)>, shader_loader: &'a ShaderLoader) -> Option<(&'a wgpu::ShaderModule, String)> {
        if let Some((label, entry)) = label_entry {
            let shader_module = shader_loader.get(label);
            Some((&shader_module.module, entry.to_string()))
        } else {
            None
        }
    }

    /// Finish building the pipeline
    pub fn finish(&self, ctx: &SystemsContext) -> Pipeline {
        let device = ctx.renderer.device();
        let shader_loader = ctx.resources.get::<ShaderLoader>().expect("ShaderLoader resource not found");
        let (vertex_module, vertex_entry) = self.load_shader(&self.vertex_shader, &shader_loader);
        let fragment_maybe = self.load_shader_maybe(&self.fragment_shader, &shader_loader);

        let color_targets = vec![self.color_format.map(|format| 
            wgpu::ColorTargetState {
                format,
                // TODO: make this field configurable
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            }
        )];

        let layout = self.bind_group_layouts.as_ref().map(|layouts| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{}_layout", self.label)),
                bind_group_layouts: &layouts.iter().collect::<Vec<_>>(), 
                push_constant_ranges: &self.push_constant_ranges,
            })
        });

        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some(&self.label),
            layout: layout.as_ref(),
            vertex: wgpu::VertexState {
                module: &vertex_module,
                entry_point: Some(&vertex_entry),
                buffers: self.vertex_buffer_layouts.as_ref(),
                compilation_options: Default::default(),
            },
            fragment: fragment_maybe.as_ref().map(|(module, entry)| wgpu::FragmentState {
                module,
                entry_point: Some(entry),
                targets: &color_targets,
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.topology.unwrap_or(wgpu::PrimitiveTopology::TriangleList),
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: self.depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        };

        let pipeline = device.create_render_pipeline(&pipeline_desc);

        Pipeline {
            inner: pipeline
        }
    }
}

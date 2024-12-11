use super::RenderHandle;

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
    pub vertex_buffer_layouts: Option<Vec<wgpu::VertexBufferLayout<'static>>>,
    pub vertex_shader: Option<(String, String)>,
    pub fragment_shader: Option<(String, String)>,
    pub color_format: Option<wgpu::TextureFormat>,
    pub depth_format: Option<wgpu::TextureFormat>,
    pub topology: Option<wgpu::PrimitiveTopology>,
}

impl PipelineBuilder {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            bind_group_layouts: None,
            vertex_buffer_layouts: None,
            vertex_shader: None,
            fragment_shader: None,
            color_format: None,
            depth_format: None,
            topology: None,
        }
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
        self.vertex_buffer_layouts = Some(layouts);
        self
    }

    /// Set wgsl vertex shader
    ///
    /// # Note
    /// Source should be a string of the shader code, you can use include_str! macro.
    /// This is required.
    pub fn set_vertex_shader(mut self, source: &str, entry_point: &str) -> Self {
        self.vertex_shader = Some((source.to_string(), entry_point.to_string()));
        self
    }

    /// Set wgsl fragment shader
    ///
    /// # Note
    /// Source should be a string of the shader code, you can use include_str! macro.
    /// If not set, the fragment state will be None
    pub fn set_fragment_shader(mut self, source: &str, entry_point: &str) -> Self {
        self.fragment_shader = Some((source.to_string(), entry_point.to_string()));
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

    fn load_shader(&self, t: &str, source_entry: &Option<(String, String)>, device: &wgpu::Device) -> (wgpu::ShaderModule, String) {
        self.load_shader_maybe(t, source_entry, device)
            .expect(&format!("{} shader for {} not set", t, self.label))
    }

    fn load_shader_maybe(&self, t: &str, source_entry: &Option<(String, String)>, device: &wgpu::Device) -> Option<(wgpu::ShaderModule, String)> {
        if let Some((source, entry)) = source_entry {
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("{}_{}_shader", self.label, t)),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

            Some((shader_module, entry.to_string()))
        } else {
            None
        }
    }

    /// Finish building the pipeline
    pub fn finish(self, device: &wgpu::Device) -> Pipeline {
        let (vertex_module, vertex_entry) = self.load_shader("vertex", &self.vertex_shader, device);
        let fragment_maybe = self.load_shader_maybe("fragment", &self.fragment_shader, device);

        let color_targets = vec![self.color_format.map(|format| 
            wgpu::ColorTargetState {
                format: format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }
        )];

        let layout = self.bind_group_layouts.map(|layouts| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{}_layout", self.label)),
                bind_group_layouts: &layouts.iter().collect::<Vec<_>>(), 
                push_constant_ranges: &[]
            })
        });

        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some(&self.label),
            layout: layout.as_ref(),
            vertex: wgpu::VertexState {
                module: &vertex_module,
                entry_point: Some(&vertex_entry),
                buffers: &self.vertex_buffer_layouts.unwrap_or(vec![]),
                compilation_options: Default::default(),
            },
            fragment: fragment_maybe.as_ref().map(|(module, entry)| wgpu::FragmentState {
                module: module,
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

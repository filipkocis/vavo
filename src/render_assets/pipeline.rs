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
    pub push_constant_ranges: Vec<wgpu::PushConstantRange>,

    /// Color targets used in the pipeline's fragment state. `fragment_shader` must be set
    pub color_targets: Vec<Option<wgpu::ColorTargetState>>,
    /// Primitive state for the pipeline.
    pub primitive_state: wgpu::PrimitiveState,
    /// Depth stencil state for the pipeline.
    pub depth_stencil: Option<wgpu::DepthStencilState>,
}

impl PipelineBuilder {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            bind_group_layouts: None,
            vertex_buffer_layouts: Vec::new(),
            vertex_shader: None,
            fragment_shader: None,
            push_constant_ranges: Vec::new(),

            color_targets: Vec::new(),
            primitive_state: Self::default_primitive_state(),
            depth_stencil: None,
        }
    }

    /// Default color target state used in the fragment field of the pipeline
    pub fn default_color_target() -> wgpu::ColorTargetState {
        wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        }
    }

    /// Default primitive state used in the pipeline
    pub fn default_primitive_state() -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        }
    }

    /// Default depth stencil state used if [`Self::set_depth_format`] is called
    pub fn default_depth_stencil() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
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

    /// Adds default ColorTargetState with `format` texture format, used in pipeline's fragment
    /// state. It's a wrapper around [`Self::add_color_target`]
    ///
    /// # Note
    /// For this to be used, you must set a fragment shader
    pub fn add_color_format(mut self, format: wgpu::TextureFormat) -> Self {
        let mut target = Self::default_color_target();
        target.format = format;
        self.color_targets.push(Some(target));
        self
    }

    /// Adds new color target state to be used in pipeline's fragment state. For default values see
    /// [`Self::default_color_target`]
    pub fn add_color_target(mut self, color_target: Option<wgpu::ColorTargetState>) -> Self {
        self.color_targets.push(color_target);
        self
    }


    /// Set texture format for pipeline's depth stencil
    ///
    /// # Note
    /// If not set, the depth stencil state will be None
    pub fn set_depth_format(mut self, depth_format: wgpu::TextureFormat) -> Self {
        if let Some(ref mut stencil) = self.depth_stencil {
            stencil.format = depth_format;
        } else {
            let mut stencil = Self::default_depth_stencil();
            stencil.format = depth_format;
            self.depth_stencil = Some(stencil);
        }
        self
    }
    
    // /// Set primitive topology for the pipeline
    // ///
    // /// # Note
    // /// Default is TriangleList
    // pub fn set_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
    //     self.primitive_state.topology = topology;
    //     self
    // }

    /// Overrides primitive state for the pipeline, for default values see [`Self::default_primitive_state`] 
    pub fn set_primitive_state(mut self, primitive_state: wgpu::PrimitiveState) -> Self {
        self.primitive_state = primitive_state;
        self
    }

    /// Overrides depth stencil state for the pipeline, for default values see [`Self::default_depth_stencil`] 
    pub fn set_depth_stencil(mut self, depth_stencil: Option<wgpu::DepthStencilState>) -> Self {
        self.depth_stencil = depth_stencil;
        self
    }

    /// Set push constant ranges for pipeline layout
    pub fn set_push_constant_ranges(mut self, ranges: Vec<wgpu::PushConstantRange>) -> Self {
        self.push_constant_ranges = ranges;
        self
    }

    fn load_shader<'a>(&self, label_entry: &Option<(String, String)>, shader_loader: &'a ShaderLoader) -> (&'a wgpu::ShaderModule, String) {
        self.load_shader_maybe(label_entry, shader_loader)
            .unwrap_or_else(|| panic!("{} shader for {} not set", label_entry.as_ref().unwrap().0, self.label))
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

        // shader modules
        let (vertex_module, vertex_entry) = self.load_shader(&self.vertex_shader, &shader_loader);
        let fragment_maybe = self.load_shader_maybe(&self.fragment_shader, &shader_loader);

        // pipeline layout
        let layout = self.bind_group_layouts.as_ref().map(|layouts| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{}_layout", self.label)),
                bind_group_layouts: &layouts.iter().collect::<Vec<_>>(), 
                push_constant_ranges: &self.push_constant_ranges,
            })
        });

        // descriptor
        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some(&self.label),
            layout: layout.as_ref(),
            vertex: wgpu::VertexState {
                module: vertex_module,
                entry_point: Some(&vertex_entry),
                buffers: self.vertex_buffer_layouts.as_ref(),
                compilation_options: Default::default(),
            },
            fragment: fragment_maybe.as_ref().map(|(module, entry)| wgpu::FragmentState {
                module,
                entry_point: Some(entry),
                targets: &self.color_targets,
                compilation_options: Default::default(),
            }),
            primitive: self.primitive_state,
            depth_stencil: self.depth_stencil.clone(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        };

        Pipeline {
            inner: device.create_render_pipeline(&pipeline_desc)
        }
    }
}

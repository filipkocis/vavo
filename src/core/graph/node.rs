use winit::dpi::PhysicalSize;

use crate::{palette, render_assets::pipeline::PipelineBuilder, system::{CustomGraphSystem, GraphSystem, SystemsContext}};

use super::{NodeColorTarget, NodeData, NodeDepthTarget};

/// Single graph node represents a render pass described by its color and depth targets, has one
/// pipeline with an execution system in a render stage. Can have multiple dependencies.
pub struct GraphNode {
    pub name: String,
    pub pipeline_builder: PipelineBuilder,
    pub system: GraphSystem,
    pub custom_system: Option<CustomGraphSystem>,
    pub color_target: NodeColorTarget,
    pub depth_target: NodeDepthTarget,
    pub color_ops: wgpu::Operations<wgpu::Color>,
    pub depth_ops: Option<wgpu::Operations<f32>>,
    /// List of dependencies
    pub after: Vec<String>,
    /// List of nodes which must render after this node
    pub before: Vec<String>,
    pub data: NodeData,
}

impl GraphNode {
    pub fn new(
        name: &str,
        pipeline_builder: PipelineBuilder,
        system: GraphSystem,
        color_target: NodeColorTarget,
        depth_target: NodeDepthTarget,
    ) -> Self {
        Self {
            name: name.to_string(),
            pipeline_builder,
            system,
            custom_system: None,
            color_target,
            depth_target,
            color_ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(palette::BLACK.into()),
                store: wgpu::StoreOp::Store,
            },
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            after: Vec::new(),
            before: Vec::new(),
            data: NodeData::new(),
        }
    }

    /// Populates the node data with the necessary data, or replaces it with new data
    pub fn generate_data(&mut self, ctx: &mut SystemsContext) {
        self.data.generate_pipeline(ctx, &self.pipeline_builder);
        self.data.generate_color_target(ctx, &self.color_target);
        self.data.generate_depth_target(ctx, &self.depth_target);

        self.data.needs_regen = false;
    }

    /// Resize the node images, currently only owned depth target is resized, 
    /// and only if color target is surface
    pub(crate) fn resize(&mut self, size: &PhysicalSize<u32>) {
        if !matches!(self.color_target, NodeColorTarget::Surface) {
            return;
        }

        if let NodeDepthTarget::Owned(image) = &mut self.depth_target {
            image.size.width = size.width;
            image.size.height = size.height;

            if let Some(texture) = &mut image.texture_descriptor {
                texture.size.width = size.width;
                texture.size.height = size.height;
            }

            self.data.needs_regen = true;
        }
    }
}

/// Helper struct to create a [`GraphNode`]
pub struct GraphNodeBuilder {
    name: String,
    pipeline_builder: Option<PipelineBuilder>,
    system: Option<GraphSystem>,
    custom_system: Option<CustomGraphSystem>,
    color_target: Option<NodeColorTarget>,
    depth_target: Option<NodeDepthTarget>,
    color_ops: wgpu::Operations<wgpu::Color>,
    depth_ops: Option<wgpu::Operations<f32>>,
    after: Vec<String>,
    before: Vec<String>,
}

impl GraphNodeBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            pipeline_builder: None,
            system: None,
            custom_system: None,
            color_target: None,
            depth_target: None,
            color_ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(palette::BLACK.into()),
                store: wgpu::StoreOp::Store,
            },
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            after: Vec::new(),
            before: Vec::new(),
        }
    }

    pub fn set_pipeline(mut self, pipeline_builder: PipelineBuilder) -> Self {
        self.pipeline_builder = Some(pipeline_builder);
        self
    }

    pub fn set_system(mut self, system: GraphSystem) -> Self {
        self.system = Some(system);
        self
    }

    /// Setting a custom system will clear the depth_ops and replace the system with
    /// an empty system. It will keep the color and depth target, if not specified they will be set
    /// to `NodeTarget::None`.
    pub fn set_custom_system(mut self, custom_system: CustomGraphSystem) -> Self {
        self.custom_system = Some(custom_system);
        self
    }

    pub fn set_color_target(mut self, color_target: NodeColorTarget) -> Self {
        self.color_target = Some(color_target);
        self
    }

    pub fn set_color_ops(mut self, ops: wgpu::Operations<wgpu::Color>) -> Self {
        self.color_ops = ops;
        self
    }

    pub fn set_depth_target(mut self, depth_target: NodeDepthTarget) -> Self {
        self.depth_target = Some(depth_target);
        self
    }

    pub fn set_depth_ops(mut self, dept_ops: Option<wgpu::Operations<f32>>) -> Self {
        self.depth_ops = dept_ops;
        self
    }

    /// Add a dependency to the node, this node will be executed after the `name` node
    pub fn run_after(mut self, name: &str) -> Self {
        if !self.after.contains(&name.to_string()) {
            self.after.push(name.to_string());
        }
        self
    }

    /// This node will be executed before the `name` node
    pub fn run_before(mut self, name: &str) -> Self {
        if !self.before.contains(&name.to_string()) {
            self.before.push(name.to_string());
        }
        self
    }

    pub fn build(mut self) -> GraphNode {
        let err = |field: &str| format!("Field '{}' for '{}' graph node is required", field, self.name);
        
        if self.custom_system.is_some() {
            let name = format!("CLEARED_{}", self.name);
            self.system = Some(GraphSystem::new(&name, |_, _, _: crate::prelude::Query<()>| {}));
            self.depth_ops = None;

            if self.color_target.is_none() {
                self.color_target = Some(NodeColorTarget::None);
            }

            if self.depth_target.is_none() {
                self.depth_target = Some(NodeDepthTarget::None);
            }
        }

        GraphNode {
            name: self.name.clone(),
            pipeline_builder: self.pipeline_builder.expect(&err("PipelineBuilder")),
            system: self.system.expect(&err("System")),
            custom_system: self.custom_system,
            color_target: self.color_target.expect(&err("ColorTarget")),
            depth_target: self.depth_target.expect(&err("DepthTarget")),
            color_ops: self.color_ops,
            depth_ops: self.depth_ops,
            after: self.after,
            before: self.before,
            data: NodeData::new(),
        }
    }
}

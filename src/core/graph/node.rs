use winit::dpi::PhysicalSize;

use crate::{render_assets::pipeline::PipelineBuilder, system::{GraphSystem, System, SystemsContext}};

use super::{NodeColorTarget, NodeData, NodeDepthTarget};

/// Single graph node represents a render pass described by its color and depth targets, has one
/// pipeline with an execution system in a render stage. Can have multiple dependencies.
pub struct GraphNode {
    pub name: String,
    pub pipeline_builder: PipelineBuilder,
    pub system: GraphSystem,
    pub color_target: NodeColorTarget,
    pub depth_target: NodeDepthTarget,
    pub dependencies: Vec<String>,
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
            color_target,
            depth_target,
            dependencies: Vec::new(),
            data: NodeData::new(),
        }
    }

    pub fn add_dependency(&mut self, dependency: &str) {
        if !self.dependencies.contains(&dependency.to_string()) {
            self.dependencies.push(dependency.to_string());
        }
    }

    pub fn remove_dependency(&mut self, dependency: &str) {
        self.dependencies.retain(|d| d != dependency);
    }

    pub fn clear_dependencies(&mut self) {
        self.dependencies.clear();
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

/// Helper struct to create a `GraphNode`
pub struct GraphNodeBuilder {
    name: String,
    pipeline_builder: Option<PipelineBuilder>,
    system: Option<GraphSystem>,
    color_target: Option<NodeColorTarget>,
    depth_target: Option<NodeDepthTarget>,
    dependencies: Vec<String>,
}

impl GraphNodeBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            pipeline_builder: None,
            system: None,
            color_target: None,
            depth_target: None,
            dependencies: Vec::new(),
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

    pub fn set_color_target(mut self, color_target: NodeColorTarget) -> Self {
        self.color_target = Some(color_target);
        self
    }

    pub fn set_depth_target(mut self, depth_target: NodeDepthTarget) -> Self {
        self.depth_target = Some(depth_target);
        self
    }

    pub fn add_dependency(mut self, dependency: &str) -> Self {
        if !self.dependencies.contains(&dependency.to_string()) {
            self.dependencies.push(dependency.to_string());
        }
        self
    }

    pub fn build(self) -> GraphNode {
        let err = |field: &str| format!("Field '{}' for '{}' graph node is required", field, self.name);

        GraphNode {
            name: self.name.clone(),
            pipeline_builder: self.pipeline_builder.expect(&err("PipelineBuilder")),
            system: self.system.expect(&err("System")),
            color_target: self.color_target.expect(&err("ColorTarget")),
            depth_target: self.depth_target.expect(&err("DepthTarget")),
            dependencies: self.dependencies,
            data: NodeData::new(),
        }
    }
}

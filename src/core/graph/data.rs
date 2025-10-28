use crate::{
    assets::ShaderLoader,
    prelude::{Texture, World},
    render_assets::{
        IntoRenderAsset, Pipeline, RenderAssetEntry, RenderAssets, pipeline::PipelineBuilder,
    },
    renderer::newtype::RenderDevice,
};

use super::{NodeColorTarget, NodeDepthTarget};

pub struct NodeData {
    pub(crate) needs_regen: bool,
    pub pipeline: Option<Pipeline>,
    pub color_target: Option<ColorTargetData>,
    pub depth_target: Option<DepthTargetData>,
}

impl Default for NodeData {
    fn default() -> Self {
        Self {
            needs_regen: true,
            pipeline: None,
            color_target: None,
            depth_target: None,
        }
    }
}

pub enum ColorTargetData {
    Texture(Texture),
    RAE(RenderAssetEntry<Texture>),
}

pub enum DepthTargetData {
    Texture(Texture),
    RAE(RenderAssetEntry<Texture>),
}

impl NodeData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_pipeline(
        &mut self,
        device: &RenderDevice,
        shader_loader: &ShaderLoader,
        pipeline_builder: &PipelineBuilder,
    ) {
        self.pipeline = Some(pipeline_builder.finish(device, shader_loader));
    }

    pub fn generate_color_target(&mut self, world: &mut World, color_target: &NodeColorTarget) {
        match color_target {
            NodeColorTarget::None => {
                self.color_target = None;
            }
            NodeColorTarget::Image(image) => {
                let mut textures = world.resources.get_mut::<RenderAssets<Texture>>();
                let texture = textures.get_by_handle(image, world);
                self.color_target = Some(ColorTargetData::RAE(texture));
            }
            NodeColorTarget::Owned(image) => {
                let target = image.create_render_asset(world, None);
                self.color_target = Some(ColorTargetData::Texture(target));
            }
            NodeColorTarget::Node(_) | NodeColorTarget::Surface => {
                // handle in the renderer
            }
        }
    }

    pub fn generate_depth_target(&mut self, world: &mut World, depth_target: &NodeDepthTarget) {
        match depth_target {
            NodeDepthTarget::None => {
                self.depth_target = None;
            }
            NodeDepthTarget::Image(image) => {
                let mut textures = world.resources.get_mut::<RenderAssets<Texture>>();
                let texture = textures.get_by_handle(image, world);
                self.depth_target = Some(DepthTargetData::RAE(texture));
            }
            NodeDepthTarget::Owned(image) => {
                let target = image.create_render_asset(world, None);
                self.depth_target = Some(DepthTargetData::Texture(target));
            }
            NodeDepthTarget::Node(_) => {
                // handle in the renderer
            }
        }
    }
}

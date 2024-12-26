use crate::{prelude::Texture, render_assets::{pipeline::PipelineBuilder, Pipeline, RenderAsset, RenderAssetEntry, RenderAssets}, system::SystemsContext};

use super::{NodeColorTarget, NodeDepthTarget};

pub struct NodeData {
    pub(crate) needs_regen: bool,
    pub pipeline: Option<Pipeline>,
    pub color_target: Option<ColorTargetData>,
    pub depth_target: Option<DepthTargetData>,
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
        Self {
            needs_regen: true,
            pipeline: None,
            color_target: None,
            depth_target: None,
        }
    }

    pub fn generate_pipeline(&mut self, ctx: &mut SystemsContext, pipeline_builder: &PipelineBuilder) {
        self.pipeline = Some(pipeline_builder.finish(ctx));
    }

    pub fn generate_color_target(&mut self, ctx: &mut SystemsContext, color_target: &NodeColorTarget) {
        match color_target {
            NodeColorTarget::None => {
                self.color_target = None;
            },
            NodeColorTarget::Image(image) => {
                let mut textures = ctx.resources.get_mut::<RenderAssets<Texture>>().expect("RenderAssets<Texture> not found");
                let texture = textures.get_by_handle(image, ctx);
                self.color_target = Some(ColorTargetData::RAE(texture));
            },
            NodeColorTarget::Owned(image) => {
                let target = image.create_render_asset(ctx, None);
                self.color_target = Some(ColorTargetData::Texture(target));
            },
            NodeColorTarget::Node(_) |
            NodeColorTarget::Surface => {
                // handle in the renderer 
            }
        }
    }

    pub fn generate_depth_target(&mut self, ctx: &mut SystemsContext, depth_target: &NodeDepthTarget) {
        match depth_target {
            NodeDepthTarget::None => {
                self.depth_target = None;
            },
            NodeDepthTarget::Image(image) => {
                let mut textures = ctx.resources.get_mut::<RenderAssets<Texture>>().expect("RenderAssets<Texture> not found");
                let texture = textures.get_by_handle(image, ctx);
                self.depth_target = Some(DepthTargetData::RAE(texture));
            },
            NodeDepthTarget::Owned(image) => {
                let target = image.create_render_asset(ctx, None);
                self.depth_target = Some(DepthTargetData::Texture(target));
            },
            NodeDepthTarget::Node(_) => {
                // handle in the renderer 
            },
        }
    }
}

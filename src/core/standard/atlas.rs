use crate::{assets::{Assets, Handle}, palette, prelude::Image, render_assets::{BindGroup, IntoRenderAsset}};

pub struct ShadowMapAtlas {
    pub image: Handle<Image>,
    pub tile_size: (u32, u32),
    pub rows: u32,
    pub cols: u32,
}

impl ShadowMapAtlas {
    pub fn new(tile_size: (u32, u32), rows: u32, cols: u32, images: &mut Assets<Image>) -> Self {
        let mut atlas = Image::new_with_defaults(vec![], wgpu::Extent3d {
            width: tile_size.0 as u32 * cols,
            height: tile_size.1 as u32 * rows,
            depth_or_array_layers: 1,
        });

        atlas.texture_descriptor.as_mut().unwrap().format = wgpu::TextureFormat::Depth32Float;
        atlas.texture_descriptor.as_mut().unwrap().view_formats = &[];
        atlas.texture_descriptor.as_mut().unwrap().usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        atlas.view_descriptor.as_mut().unwrap().format = Some(wgpu::TextureFormat::Depth32Float);
        atlas.sampler_descriptor.as_mut().unwrap().compare = Some(wgpu::CompareFunction::LessEqual);

        let image = images.add(atlas);
    
        Self {
            image,
            tile_size,
            rows,
            cols,
        }
    }
}

impl IntoRenderAsset<BindGroup> for ShadowMapAtlas {
    fn create_render_asset(
        &self, 
        ctx: &mut crate::prelude::SystemsContext,
        _: Option<&crate::prelude::EntityId>
    ) -> BindGroup {
        BindGroup::build("shadow_map_atlas")
            .add_texture(
                &Some(self.image.clone()), 
                ctx, 
                palette::BLACK, 
                Some(wgpu::TextureSampleType::Depth),
                Some(wgpu::SamplerBindingType::Comparison),
            )
            .finish(ctx)
    }
}

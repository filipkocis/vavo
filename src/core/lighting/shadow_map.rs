use crate::{
    prelude::{Image, World},
    render_assets::IntoRenderAsset,
};

/// Single texture array with a sampler
pub struct ShadowMapArray {
    pub size: wgpu::Extent3d,
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
}

impl ShadowMapArray {
    pub fn new(world: &mut World, size: wgpu::Extent3d) -> Self {
        let image = Image {
            data: Vec::new(),
            size,
            texture_descriptor: Some(wgpu::TextureDescriptor {
                label: Some("ShadowMapArray Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[wgpu::TextureFormat::Depth32Float],
            }),
            sampler_descriptor: Some(wgpu::SamplerDescriptor {
                label: Some("ShadowMapArray Sampler"),
                compare: Some(wgpu::CompareFunction::LessEqual),
                ..Default::default()
            }),
            // We don't need a view, but we need to specify the format or Image will use
            // incompatible defaults which will cause a panic
            view_descriptor: Some(wgpu::TextureViewDescriptor {
                label: Some("ShadowMapArray View"),
                format: Some(wgpu::TextureFormat::Depth32Float),
                ..Default::default()
            }),
        };

        let texture = image.create_render_asset(world, None);

        Self {
            size,
            texture: texture.texture,
            sampler: texture.sampler,
        }
    }

    /// Resize the texture array to n layers. Panics if `n == 0`
    pub fn resize(&mut self, world: &mut World, layers: u32) {
        assert!(layers > 0);

        if layers == self.layers() {
            return;
        }

        self.size.depth_or_array_layers = layers;
        let resized = Self::new(world, self.size);

        self.texture = resized.texture;
    }

    /// Get the size of the `depth_or_array_layers`
    pub fn layers(&self) -> u32 {
        self.size.depth_or_array_layers
    }

    /// Create a texture view for this shadow map array
    pub fn create_view(
        &self,
        layer: u32,
        count: Option<u32>,
        dimension: Option<wgpu::TextureViewDimension>,
    ) -> wgpu::TextureView {
        self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!(
                "ShadowMapArray View [{}..{}]",
                layer,
                count
                    .map(|c| layer + c)
                    .unwrap_or(self.size.depth_or_array_layers)
            )),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension,
            aspect: wgpu::TextureAspect::DepthOnly,
            base_array_layer: layer,
            array_layer_count: count,
            ..Default::default()
        })
    }
}

// TODO: cant create RAE because after resizing it will be invalid

// impl RenderAsset<BindGroup> for ShadowMapArray {
//     fn create_render_asset(
//         &self,
//         ctx: &mut SystemsContext,
//         _: Option<&crate::prelude::EntityId>
//     ) -> BindGroup {
//         BindGroup::build("shadow_map_array")
//             .add_custom(
//                 wgpu::ShaderStages::FRAGMENT,
//                 wgpu::BindingType::Texture {
//                     sample_type: wgpu::TextureSampleType::Depth,
//                     view_dimension: wgpu::TextureViewDimension::D2Array,
//                     multisampled: false,
//                 },
//                 None,
//                 wgpu::BindingResource::TextureView(&self.create_view(0, None, None))
//             )
//             .add_custom(
//                 wgpu::ShaderStages::FRAGMENT,
//                 wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
//                 None,
//                 wgpu::BindingResource::Sampler(&self.sampler)
//             )
//             .finish(ctx)
//     }
// }

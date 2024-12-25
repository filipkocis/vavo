use crate::{assets::Assets, render_assets::{RenderAsset, RenderAssetEntry, RenderAssets}, system::SystemsContext};

use super::Color;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

#[derive(Clone)]
/// Texture render asset which represents a 1x1 texture with a single rgba color. 
/// Created with default image descriptors.
///
/// Internally, it creates a new image asset and creates a render asset texture from it
pub struct SingleColorTexture {
    pub handle: RenderAssetEntry<Texture>,
}

impl SingleColorTexture {
    pub fn new(ctx: &mut SystemsContext, color: Color) -> Self {
        let image = Image {
            data: color.as_rgba_slice_u8().to_vec(),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            texture_descriptor: None,
            sampler_descriptor: None,
            view_descriptor: None,
        };

        let mut images = ctx.resources.get_mut::<Assets<Image>>().unwrap();
        let image = images.add(image);

        let mut textures = ctx.resources.get_mut::<RenderAssets<Texture>>().unwrap();
        let texture = textures.get_by_handle(&image, ctx);

        // TODO add optimization to not create a new texture if similar texture already exists

        Self {
            handle: texture,
        }
    }
}

pub struct Image {
    /// Image data, if set, will be used to write to the texture during creation
    pub data: Vec<u8>,
    pub size: wgpu::Extent3d,
    pub texture_descriptor: Option<wgpu::TextureDescriptor<'static>>,
    pub sampler_descriptor: Option<wgpu::SamplerDescriptor<'static>>,
    pub view_descriptor: Option<wgpu::TextureViewDescriptor<'static>>,
}

impl Image {
    pub fn new_with_defaults(data: Vec<u8>, size: wgpu::Extent3d) -> Self {
        Self {
            data,
            size,
            texture_descriptor: Some(Self::default_texture_descriptor(size)),
            sampler_descriptor: Some(Self::default_sampler_descriptor()),
            view_descriptor: Some(Self::default_view_descriptor()),
        }
    }

    pub fn default_texture_descriptor(size: wgpu::Extent3d) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some("Image Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        }
    }

    pub fn default_sampler_descriptor() -> wgpu::SamplerDescriptor<'static> {
        wgpu::SamplerDescriptor {
            label: Some("Image Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        }
    }
    
    pub fn default_view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
        wgpu::TextureViewDescriptor {
            label: Some("Image Texture View"),
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()  
        }
    }
}

impl RenderAsset<Texture> for Image {
    fn create_render_asset(
        &self, 
        ctx: &mut crate::prelude::SystemsContext,
        _: Option<&crate::prelude::EntityId>
    ) -> Texture {
        let device = ctx.renderer.device();
        let queue = ctx.renderer.queue();

        let default_texture_descriptor = Self::default_texture_descriptor(self.size);
        let texture_descriptor = self.texture_descriptor.as_ref().unwrap_or(&default_texture_descriptor);

        let texture = device.create_texture(texture_descriptor);
        let view = texture.create_view(self.view_descriptor.as_ref().unwrap_or(&Self::default_view_descriptor()));
        let sampler = device.create_sampler(self.sampler_descriptor.as_ref().unwrap_or(&Self::default_sampler_descriptor()));

        if !self.data.is_empty() {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.size.width),
                    rows_per_image: Some(self.size.height),
                },
                self.size,
            );
        }

        Texture {
            texture,
            view,
            sampler,
        }
    }
}

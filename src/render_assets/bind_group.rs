use std::num::NonZero;

use crate::prelude::Color;
use crate::renderer::{Image, SingleColorTexture, Texture};
use crate::assets::Handle;
use crate::system::SystemsContext;

use super::render_assets::RenderAssetEntry;
use super::RenderAssets;

#[derive(crate::macros::RenderAsset)]
pub struct BindGroup {
    pub(crate) inner: wgpu::BindGroup,
}

impl<'a> From<&'a BindGroup> for Option<&'a wgpu::BindGroup> {
    fn from(value: &'a BindGroup) -> Self {
        Some(&value.inner)
    }
}

impl BindGroup {
    pub fn build<'a>(label: &'a str) -> BindGroupBuilder<'a> {
        BindGroupBuilder::new(label)
    }
}

pub struct BindGroupBuilder<'a> {
    label: &'a str,
    layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    entries: Vec<wgpu::BindGroupEntry<'a>>,

    textures: Vec<(u32, RenderAssetEntry<Texture>, Option<wgpu::TextureSampleType>, Option<wgpu::SamplerBindingType>)>,
    binding: u32,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            layout_entries: Vec::new(),
            entries: Vec::new(),

            textures: Vec::new(),
            binding: 0,
        }
    }

     pub fn add_custom(mut self, visibility: wgpu::ShaderStages, ty: wgpu::BindingType, count: Option<u32>, resource: wgpu::BindingResource<'a>) -> Self {
        let layout_entry = wgpu::BindGroupLayoutEntry {
            binding: self.binding,
            visibility,
            ty,
            count: count.map(|c| NonZero::new(c).expect("Count must be greater than zezro")),
        };

        let entry = wgpu::BindGroupEntry {
            binding: layout_entry.binding,
            resource,
        };

        self.layout_entries.push(layout_entry);
        self.entries.push(entry);
        self.binding += 1;
        self
    }


    pub fn add_texture(mut self, texture: &Option<Handle<Image>>, ctx: &mut SystemsContext, default_color: Color, sample_type: Option<wgpu::TextureSampleType>, sampler_bind: Option<wgpu::SamplerBindingType>) -> Self {
        if let Some(texture) = texture {
            let mut render_images = ctx.resources.get_mut::<RenderAssets<Texture>>().unwrap();
            let texture = render_images.get_by_handle(texture, ctx);
            self.textures.push((self.binding, texture, sample_type, sampler_bind));
        } else {
            let default_texture = SingleColorTexture::new(ctx, default_color).handle;
            self.textures.push((self.binding, default_texture, sample_type, sampler_bind)); 
        }

        self.binding += 2;
        self
    }

    pub fn add_uniform_buffer(mut self, buffer: &'a wgpu::Buffer, visibility: wgpu::ShaderStages) -> Self {
        let ty = wgpu::BufferBindingType::Uniform;
        self.add_buffer(buffer, visibility, ty);
        self
    }

    pub fn add_storage_buffer(mut self, buffer: &'a wgpu::Buffer, visibility: wgpu::ShaderStages, read_only: bool) -> Self {
        let ty = wgpu::BufferBindingType::Storage { read_only };
        self.add_buffer(buffer, visibility, ty);
        self
    }

    fn add_buffer(&mut self, buffer: &'a wgpu::Buffer, visibility: wgpu::ShaderStages, ty: wgpu::BufferBindingType) {
        let layout_entry = wgpu::BindGroupLayoutEntry {
            binding: self.binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let entry = wgpu::BindGroupEntry {
            binding: layout_entry.binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: None,
            })
        };

        self.layout_entries.push(layout_entry);                             
        self.entries.push(entry);
        self.binding += 1;
    }

    fn texture_layout_entries(&self) -> (Vec<wgpu::BindGroupEntry>, Vec<wgpu::BindGroupLayoutEntry>) {
        let mut layouts = Vec::new();
        let mut entries = Vec::new();

        for (binding, texture, sample_type, sampler_bind) in &self.textures {
            let tle = wgpu::BindGroupLayoutEntry {
                binding: *binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: sample_type.unwrap_or(wgpu::TextureSampleType::Float { filterable: true }),
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            };

            let te = wgpu::BindGroupEntry {
                binding: tle.binding,
                resource: wgpu::BindingResource::TextureView(&texture.view)
            };

            layouts.push(tle);
            entries.push(te);

            let sle = wgpu::BindGroupLayoutEntry {
                binding: binding + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(sampler_bind.unwrap_or(wgpu::SamplerBindingType::Filtering)),
                count: None,
            };

            let se = wgpu::BindGroupEntry {
                binding: sle.binding,
                resource: wgpu::BindingResource::Sampler(&texture.sampler)
            };

            layouts.push(sle);
            entries.push(se);
        }
        
        (entries, layouts)
    }

    pub fn finish(self, ctx: &mut SystemsContext) -> BindGroup {
        let (mut entries, mut layouts) = self.texture_layout_entries();
        
        layouts.extend(self.layout_entries.clone());
        entries.extend(self.entries.clone());

        let device = ctx.renderer.device();

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &layouts,
            label: Some(&format!("{}_bind_group_layout", self.label))
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &entries,
            label: Some(&format!("{}_bind_group", self.label))
        });

        BindGroup {
            inner: bind_group,
        }
    }
}

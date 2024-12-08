use crate::renderer::Image;
use crate::resources::Resources;
use crate::assets::{Assets, Handle};

use super::RenderAssets;

pub struct BindGroup {
    pub(crate) inner: wgpu::BindGroup,
}

impl<'a> Into<Option<&'a wgpu::BindGroup>> for &'a BindGroup {
    fn into(self) -> Option<&'a wgpu::BindGroup> {
        Some(&self.inner)
    }
}

impl BindGroup {
    pub fn build<'a>(label: &'a str, device: &'a wgpu::Device) -> BindGroupBuilder<'a> {
        BindGroupBuilder::new(label, device)
    }
}

pub struct BindGroupBuilder<'a> {
    label: &'a str,
    device: &'a wgpu::Device,
    layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new(label: &'a str, device: &'a wgpu::Device) -> Self {
        Self {
            label,
            device,
            layout_entries: Vec::new(),
            entries: Vec::new(),
        }
    }

    pub fn add_texture(mut self, texture: &Option<Handle<Image>>, resources: &mut Resources) -> Self {
        if let Some(texture) = texture {
            todo!();
            // let images = resources.get::<Assets<Image>>().unwrap();
            // let render_images = resources.get::<RenderAssets<Image>>().unwrap();
            // let texture = render_images.get(texture, &images).unwrap();
            //
            // let layout_entry = wgpu::BindGroupLayoutEntry {
            //     binding: self.entries.len() as u32,
            //     visibility: wgpu::ShaderStages::FRAGMENT,
            //     ty: wgpu::BindingType::Texture {
            //         sample_type: wgpu::TextureSampleType::Float { filterable: true },
            //         view_dimension: wgpu::TextureViewDimension::D2,
            //         multisampled: false,
            //     },
            //     count: None,
            // };
            //
            // let entry = wgpu::BindGroupEntry {
            //     binding: layout_entry.binding,
            //     resource: wgpu::BindingResource::TextureView(&texture.view),
            // };
        }

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
            binding: self.layout_entries.len() as u32,
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
    }

    pub fn finish(self) -> BindGroup {
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &self.layout_entries,
            label: Some(&format!("{}_bind_group_layout", self.label))
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &self.entries,
            label: Some(&format!("{}_bind_group", self.label))
        });

        BindGroup {
            inner: bind_group,
        }
    }
}

use crate::{assets::Handle, prelude::Resources, render_assets::{BindGroup, Buffer, RenderAsset}, world::EntityId};

use super::{Color, Face, Image};

pub struct Material {
    pub base_color: Color,
    pub base_color_texture: Option<Handle<Image>>,
    pub normal_map_texture: Option<Handle<Image>>,

    pub emissive: Color,
    pub emissive_exposure_weight: f32,

    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,

    pub flip_normal_map_y: bool,
    pub cull_mode: Option<Face>,
    pub unlit: bool,
}

impl Material {
    fn uniform_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(bytemuck::bytes_of(&self.base_color));
        data.extend_from_slice(bytemuck::bytes_of(&self.emissive));
        data.extend_from_slice(bytemuck::cast_slice(&[
            self.emissive_exposure_weight,
            self.perceptual_roughness,
            self.metallic,
            self.reflectance,
        ]));
        data.extend_from_slice(bytemuck::cast_slice(&[
            self.flip_normal_map_y,
            matches!(self.cull_mode, Some(Face::Back)),
            self.unlit
        ]));

        data
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: Color::default(),
            base_color_texture: None,
            normal_map_texture: None,
            emissive: Color::rgb(0.0, 0.0, 0.0),
            emissive_exposure_weight: 1.0,
            perceptual_roughness: 0.5,
            metallic: 0.5,
            reflectance: 0.5,
            flip_normal_map_y: false,
            cull_mode: Some(Face::Back),
            unlit: false,
        } 
    }
}

impl RenderAsset<Buffer> for Material {
    fn create_render_asset(
        &self, 
        device: 
        &wgpu::Device, 
        _: &mut Resources,
        _: Option<&EntityId>
    ) -> Buffer {
        Buffer::new("material")
            .create_uniform_buffer(&self.uniform_data(), None, device)
    }
}

impl RenderAsset<BindGroup> for Material {
    fn create_render_asset(
        &self, 
        device: 
        &wgpu::Device, 
        resources: &mut Resources,
        _: Option<&EntityId>
    ) -> BindGroup {
        BindGroup::build("material", device)
            .add_texture(&self.base_color_texture, resources)
            .add_texture(&self.normal_map_texture, resources)
            // .add_uniform(self, device)
            .finish()
    }
}

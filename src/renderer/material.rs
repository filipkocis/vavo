use crate::{assets::Handle, render_assets::{BindGroup, Buffer, IntoRenderAsset}, system::SystemsContext, ecs::entities::EntityId};

use super::{palette, Color, Face, Image};

#[derive(Debug, Clone, crate::macros::Asset)]
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


        let booleans = self.flip_normal_map_y as u32 |
            ((matches!(self.cull_mode, Some(Face::Back)) as u32) << 1) |
            ((self.unlit as u32) << 2);
        data.extend_from_slice(bytemuck::cast_slice(&[
            booleans, 0, 0, 0
        ]));

        data
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: palette::WHITE,
            base_color_texture: None,
            normal_map_texture: None,
            emissive: Color::rgb(0.0, 0.0, 0.0),
            emissive_exposure_weight: 1.0,
            perceptual_roughness: 0.4,
            metallic: 0.0,
            reflectance: 0.04,
            flip_normal_map_y: false,
            cull_mode: Some(Face::default()),
            unlit: false,
        } 
    }
}

impl IntoRenderAsset<Buffer> for Material {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        _: Option<EntityId>
    ) -> Buffer {
        Buffer::new("material")
            .create_uniform_buffer(&self.uniform_data(), None, ctx.renderer.device())
    }
}

impl IntoRenderAsset<BindGroup> for Material {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        _: Option<EntityId>
    ) -> BindGroup {
        let buffer: Buffer = self.create_render_asset(ctx, None);
        let uniform = buffer.uniform.expect("Material buffer should be an uniform buffer");

        BindGroup::build("material")
            .add_texture(&self.base_color_texture, ctx, self.base_color, None, None)
            .add_texture(&self.normal_map_texture, ctx, Color::rgb(0.5, 0.5, 1.0), None, None)
            .add_uniform_buffer(&uniform, wgpu::ShaderStages::VERTEX_FRAGMENT)
            .finish(ctx)
    }
}

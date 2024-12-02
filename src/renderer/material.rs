use crate::assets::Handle;

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

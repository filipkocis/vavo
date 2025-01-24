use crate::{prelude::Light, render_assets::{BindGroup, LightStorage, RenderAsset}, system::SystemsContext};

use super::ShadowMapArray;

/// Manages the light storage and shadow maps for every applicable light type
pub struct LightAndShadowManager {
    pub storage: LightStorage,
    directional_shadow_map: ShadowMapArray,
    point_shadow_map: ShadowMapArray,
    spot_shadow_map: ShadowMapArray,
    sampler: wgpu::Sampler,
}

impl LightAndShadowManager {
    pub const LIGHT_SIZE: usize = std::mem::size_of::<Light>();
    pub const DIRECTIONAL_SHADOW_MAP_SIZE: u32 = 2048;
    pub const POINT_SHADOW_MAP_SIZE: u32 = 512;
    pub const SPOT_SHADOW_MAP_SIZE: u32 = 1024;
    pub const SHADOW_MAP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(ctx: &mut SystemsContext) -> Self {
        let storage = LightStorage::new(1, Self::LIGHT_SIZE, ctx, wgpu::ShaderStages::VERTEX_FRAGMENT);

        let directional_shadow_map = ShadowMapArray::new(ctx, wgpu::Extent3d {
            width: Self::DIRECTIONAL_SHADOW_MAP_SIZE,
            height: Self::DIRECTIONAL_SHADOW_MAP_SIZE,
            depth_or_array_layers: 1 
        });

        let point_shadow_map = ShadowMapArray::new(ctx, wgpu::Extent3d {
            width: Self::POINT_SHADOW_MAP_SIZE,
            height: Self::POINT_SHADOW_MAP_SIZE,
            depth_or_array_layers: 6
        });

        let spot_shadow_map = ShadowMapArray::new(ctx, wgpu::Extent3d {
            width: Self::SPOT_SHADOW_MAP_SIZE,
            height: Self::SPOT_SHADOW_MAP_SIZE,
            depth_or_array_layers: 1
        });

        let sampler = ctx.renderer.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("LightAndShadowManager Sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            storage,
            directional_shadow_map,
            point_shadow_map,
            spot_shadow_map,
            sampler,
        }
    }

    /// Update the light storage and shadow maps to match the lights.
    /// Sets the shadow map index for each light.
    pub fn update(&mut self, lights: &mut [Light], ctx: &mut SystemsContext) {
        let mut directional_lights = 0u32;
        let mut point_lights = 0u32;
        let mut spot_lights = 0u32;

        for light in lights.iter_mut() {
            if light.is_directional() {
                light.set_shadow_map_index(directional_lights);
                directional_lights += 1;
            } else if light.is_point() {
                light.set_shadow_map_index(point_lights);
                point_lights += 1;
            } else if light.is_spot() {
                light.set_shadow_map_index(spot_lights);
                spot_lights += 1;
            }
        } 

        assert_eq!(point_lights % 6, 0, "Point lights must be a multiple of 6, one for each cube face");

        directional_lights = directional_lights.max(1);
        point_lights = point_lights.max(6);
        spot_lights = spot_lights.max(1);

        self.directional_shadow_map.resize(ctx, directional_lights);
        self.point_shadow_map.resize(ctx, point_lights);
        self.spot_shadow_map.resize(ctx, spot_lights);
        
        self.storage.update(&lights, lights.len(), ctx);
    }

    pub fn create_view(&self, light: &Light) -> wgpu::TextureView {
        let layer = light.shadow_map_index();
        let count = Some(1);
        let dimension = Some(wgpu::TextureViewDimension::D2);

        if light.is_directional() {
            self.directional_shadow_map.create_view(layer, count, dimension)
        } else if light.is_point() {
            self.point_shadow_map.create_view(layer, count, dimension)
        } else if light.is_spot() {
            self.spot_shadow_map.create_view(layer, count, dimension)
        } else {
            panic!("Could not create view, light type not supported for shadow mapping");
        }
    }

    /// Create a texture view for the shadow map at any given index, with light type: 
    /// 0 - Directional, 1 - Point, 2 - Spot
    ///
    /// # Note
    /// Unsafe primarily to discourage use of this method, use `create_view` with lights created
    /// from `PreparedLightData` resource.
    ///
    /// # Safety
    /// Does ont check if shadow map index is valid
    pub unsafe fn unsafe_create_view(&self, shadow_map_index: u32, light_type: u32) -> wgpu::TextureView {
        let layer = shadow_map_index;
        let count = Some(1);
        let dimension = Some(wgpu::TextureViewDimension::D2);

        match light_type {
            0 => self.directional_shadow_map.create_view(layer, count, dimension),
            1 => self.point_shadow_map.create_view(layer, count, dimension),
            2 => self.spot_shadow_map.create_view(layer, count, dimension),
            _ => panic!("Could not create view, invalid light type: {:#?}", light_type) 
        }
    }
}

impl IntoRenderAsset<BindGroup> for LightAndShadowManager {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        _: Option<&crate::prelude::EntityId>
    ) -> BindGroup {
        let visibility = wgpu::ShaderStages::FRAGMENT;
        let binding_type = wgpu::BindingType::Texture { 
            sample_type: wgpu::TextureSampleType::Depth,
            view_dimension: wgpu::TextureViewDimension::D2Array, 
            multisampled: false 
        };
        let cube_binding_type = wgpu::BindingType::Texture { 
            sample_type: wgpu::TextureSampleType::Depth,
            view_dimension: wgpu::TextureViewDimension::CubeArray, 
            multisampled: false 
        };

        BindGroup::build("LightAndShadowManager") 
            .add_storage_buffer(self.storage.buffer(), wgpu::ShaderStages::VERTEX_FRAGMENT, true)
            .add_custom(visibility, binding_type, None, 
                wgpu::BindingResource::TextureView(&self.directional_shadow_map.create_view(0, None, Some(wgpu::TextureViewDimension::D2Array)))
            )
            .add_custom(visibility, cube_binding_type, None, 
                wgpu::BindingResource::TextureView(&self.point_shadow_map.create_view(0, None, Some(wgpu::TextureViewDimension::CubeArray)))
            )
            .add_custom(visibility, binding_type, None, 
                wgpu::BindingResource::TextureView(&self.spot_shadow_map.create_view(0, None, Some(wgpu::TextureViewDimension::D2Array)))
            )
            .add_custom(
                visibility, 
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), 
                None,
                wgpu::BindingResource::Sampler(&self.sampler)
            )
            .finish(ctx)
    }
}

use std::{any::Any, collections::HashMap, fmt::Debug, path::Path};

use crate::prelude::{Color, Image, Material, Mesh, Resources};

use super::{Asset, Assets, Handle};

#[derive(crate::macros::Resource)]
pub struct AssetLoader {
    /// Cache of loaded assets, stores Handle<T: LoadableAsset>
    cache: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl AssetLoader {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new()
        }
    }

    pub fn load<A: LoadableAsset>(&mut self, path: &str, resources: &mut Resources) -> Handle<A> {
        if let Some(handle) = self.cache.get(path) {
            return handle.downcast_ref::<Handle<A>>()
                .unwrap_or_else(|| panic!("Could not downcast asset handle for '{}'", path))
                .clone();
        }

        let asset = A::load(self, resources, path);
        let mut assets = resources.try_get_mut::<Assets<A>>()
            .unwrap_or_else(|| panic!("Could not find Assets<A> in resources when loading '{}'", path));

        let handle = assets.add(asset);
        self.cache.insert(path.to_string(), Box::new(handle.clone()));
        
        handle
    }
}

/// Trait for assets which can be loaded from a file
pub trait LoadableAsset: Asset {
    fn load<P: AsRef<Path> + Debug>(loader: &mut AssetLoader, resources: &mut Resources, path: P) -> Self;
}

impl LoadableAsset for Material {
    fn load<P: AsRef<Path> + Debug>(loader: &mut AssetLoader, resources: &mut Resources, path: P) -> Self {
        let (obj_materials, _) = tobj::load_mtl(path.as_ref())
            .unwrap_or_else(|_| panic!("Could not load mtl file at '{:?}'", path));

        let mut full_path = std::fs::canonicalize(path.as_ref()).unwrap_or_else(|_| panic!("Could not cannonicalize path '{:?}'", path));

        let mut get_path = |path: &str| {
            full_path.pop();
            full_path.push(path);
            full_path.to_str().expect("Could not convert path to string").to_string()
        };

        let mut materials: Vec<Material> = Vec::new();
        for mat in obj_materials {
            let material = Material {
                base_color: mat.diffuse.map(|c| Color::from_rgb_slice(&c)).unwrap_or_default(),
                base_color_texture: mat.diffuse_texture.map(|path| loader.load(&get_path(path.as_ref()), resources)),
                // TODO: check with learnwgpu how to handle normal_texture
                normal_map_texture: mat.normal_texture.map(|path| loader.load(&get_path(path.as_ref()), resources)),
                perceptual_roughness: mat.shininess.map(|s| (1.0 - s / 100.0).clamp(0.0, 1.0)).unwrap_or(0.5),
                unlit: mat.illumination_model == Some(0),
                ..Default::default()
            };

            materials.push(material)
        }

        if materials.len() > 1 {
            // TODO: handle multiple materials in mtl file
            unimplemented!("Multiple materials in mtl file at '{:?}'", path);
        }

        match materials.len() == 1 {
            true => materials.remove(0),
            false => Material::default()
        }
    }
}

impl LoadableAsset for Mesh {
    fn load<P: AsRef<Path> + Debug>(_: &mut AssetLoader, _: &mut Resources, path: P) -> Self {
        let (models, _) = tobj::load_obj(path.as_ref(), &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        }).unwrap_or_else(|_| panic!("Could not load obj file at '{:?}'", path));
 
        if models.len() > 1 {
            // TODO: handle multiple models in obj file
            unimplemented!("Multiple models in obj file at '{:?}'", path);
        }
        let model = models.into_iter().next().unwrap_or_else(|| panic!("No models found in obj file at '{:?}'", path));
        let model_mesh = model.mesh;

        let mut colors = Vec::new();
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();

        for i in 0..model_mesh.positions.len() / 3 {
            positions.push([
                model_mesh.positions[i * 3],
                model_mesh.positions[i * 3 + 1],
                model_mesh.positions[i * 3 + 2],
            ]);

            if !model_mesh.normals.is_empty() {
                normals.push([
                    model_mesh.normals[i * 3],
                    model_mesh.normals[i * 3 + 1],
                    model_mesh.normals[i * 3 + 2],
                ])
            }

            if !model_mesh.texcoords.is_empty() {
                uvs.push([
                    model_mesh.texcoords[i * 2],
                    // TODO: check if we need to flip the y coordinate
                    1.0 - model_mesh.texcoords[i * 2 + 1],
                ])
            }

            if !model_mesh.vertex_color.is_empty() {
                colors.push(Color::rgb(
                    model_mesh.vertex_color[i * 3],
                    model_mesh.vertex_color[i * 3 + 1],
                    model_mesh.vertex_color[i * 3 + 2],
                ))
            }
        }

        Mesh {
            topology: wgpu::PrimitiveTopology::TriangleList,
            colors: if colors.is_empty() { None } else { Some(colors) },
            positions,
            normals: if normals.is_empty() { None } else { Some(normals) },
            uvs: if uvs.is_empty() { None } else { Some(uvs) },
            indices: if model_mesh.indices.is_empty() { None } else { Some(model_mesh.indices) },
        }
    }
}

impl LoadableAsset for Image {
    fn load<P: AsRef<Path> + Debug>(_: &mut AssetLoader, _: &mut Resources, path: P) -> Self {
        let image = image::open(path.as_ref())
            .unwrap_or_else(|_| panic!("Could not open image at '{:?}'", path))
            .to_rgba8();

        let (width, height) = image.dimensions();
        let data = image.into_raw();
        let size = wgpu::Extent3d {
            width, height,
            depth_or_array_layers: 1,
        };
        
        Image::new_with_defaults(data, size)
    }
}

use std::collections::HashMap;

use wgpu::{Device, ShaderSource};

pub struct Shader {
    /// Label used in the shader module will be `label_shader`.
    /// e.g. label: "main" -> main_shader
    pub label: String,
    pub module: wgpu::ShaderModule,
}

impl Shader {
    pub fn new(device: &Device, label: &str, source: ShaderSource) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{}_shader", label)),
            source,
        });

        Self {
            label: label.to_string(),
            module,
        }
    }

    pub fn wgsl(device: &Device, label: &str, source: &str) -> Self {
        let source = ShaderSource::Wgsl(source.into());
        Self::new(device, label, source)
    }
}

/// Cache storage for shader modules, use ShaderLoader::load to load new shader, and
/// ShaderLoader::get to get a shader module by label
///
/// # Info
/// This may be removed in the future when a more robust system is in place, currently doing it
/// with AssetLoader will not work
#[derive(crate::macros::Resource)]
pub struct ShaderLoader {
    cache: HashMap<String, Shader>,
}

impl ShaderLoader {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new()
        }
    }

    /// Load and creates a wgsl shader, returns None if label already exists.
    /// Source is a string of a wgsl shader code, you can use include_str! macro.
    pub fn load(&mut self, label: &str, wgsl: &str, device: &Device) -> Option<&Shader> {
        if self.cache.contains_key(label) {
            return None;
        }

        let shader = Shader::wgsl(device, label, wgsl);
        self.cache.insert(label.to_string(), shader);

        Some(self.cache.get(label).expect("Shader label should exist after insertion"))
    }

    /// Get a shader by label
    pub fn get(&self, label: &str) -> &Shader {
        if let Some(shader) = self.cache.get(label) {
            shader
        } else {
            panic!("Shader with label '{}' does not exist", label);
        }
    }
}

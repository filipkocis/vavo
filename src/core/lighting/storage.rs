use std::ops::{Deref, DerefMut};

use crate::{render_assets::Storage, system::SystemsContext};

#[derive(crate::macros::Resource)]
/// Storage for light data, used in the light manager
pub struct LightStorage(Storage);

impl LightStorage {
    pub fn new(n: usize, size: usize, ctx: &mut SystemsContext, visibility: wgpu::ShaderStages) -> Self {
        Self(Storage::new("light", n, size, ctx, visibility))
    }
}

impl Deref for LightStorage {
    type Target = Storage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LightStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

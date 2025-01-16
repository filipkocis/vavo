use std::ops::{Deref, DerefMut};

use crate::{render_assets::Storage, system::SystemsContext};

pub struct UiTransformStorage(Storage);

impl UiTransformStorage {
    pub fn new(n: usize, size: usize, ctx: &mut SystemsContext, visibility: wgpu::ShaderStages) -> Self {
        Self(Storage::new("node_transform", n, size, ctx, visibility))
    }
}

impl Deref for UiTransformStorage {
    type Target = Storage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UiTransformStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

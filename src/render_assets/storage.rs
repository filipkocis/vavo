use std::ops::{Deref, DerefMut};

use bytemuck::{AnyBitPattern, NoUninit};

use crate::system::SystemsContext;

use super::{BindGroup, Buffer};

/// Special kind of RenderAsset which is stored as a global Resource
/// Contains a buffer with instanced transform data, and the bind group
pub struct Storage {
    name: String,
    /// Size of the buffer in bytes
    size: usize,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl Storage {
    /// Create a new Storage with n transforms of size bytes
    pub fn new(name: &str, n: usize, size: usize, ctx: &mut SystemsContext) -> Self {
        let data = vec![0u8; n * size];

        let buffer = Buffer::new("transform_storage")
            .create_storage_buffer(&data, Some(wgpu::BufferUsages::COPY_DST), ctx.renderer.device());

        let storage_buffer = buffer.storage.as_ref()
            .expect("Storage buffer should be storage");
        let bind_group = BindGroup::build(&format!("{}_storage", name))
            .add_storage_buffer(storage_buffer, wgpu::ShaderStages::VERTEX, true)
            .finish(ctx);

        Self { name: name.to_string(), buffer, bind_group, size: n * size }
    }

    /// Set new size for the buffer
    pub fn resize(&mut self, n: usize, size: usize, ctx: &mut SystemsContext) {
        if n * size == self.size {
            return;
        }

        let new = Self::new(&self.name, n, size, ctx);

        self.buffer = new.buffer;
        self.bind_group = new.bind_group;
    }

    /// Update the buffer with new data
    /// Resizes the buffer if the data is larger than the current buffer size
    pub fn update<A>(&mut self, data: &[A], ctx: &mut SystemsContext) 
    where A: NoUninit + AnyBitPattern 
    {
        let data = bytemuck::cast_slice(data);

        if data.len() > self.size {
            self.resize(data.len(), 1, ctx);
        }

        let buffer = self.buffer();
        ctx.renderer.queue().write_buffer(buffer, 0, data);
    }

    /// Return the storage buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        self.buffer.storage.as_ref().expect("Storage buffer should be storage")
    }

    /// Return the bind group
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group.inner
    }
}

pub struct TransformStorage(Storage);
pub struct LightStorage(Storage);

impl TransformStorage {
    pub fn new(n: usize, size: usize, ctx: &mut SystemsContext) -> Self {
        Self(Storage::new("transform", n, size, ctx))
    }
}

impl LightStorage {
    pub fn new(n: usize, size: usize, ctx: &mut SystemsContext) -> Self {
        Self(Storage::new("light", n, size, ctx))
    }
}

impl Deref for TransformStorage {
    type Target = Storage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for LightStorage {
    type Target = Storage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TransformStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DerefMut for LightStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

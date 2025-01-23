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
    /// Amount of elements in the buffer
    count: usize,
    buffer: Buffer,
    bind_group: BindGroup,
    visibility: wgpu::ShaderStages,
}

impl Storage {
    /// Create a new Storage with n transforms of size bytes
    pub fn new(name: &str, count: usize, element_size: usize, ctx: &mut SystemsContext, visibility: wgpu::ShaderStages) -> Self {
        let data = vec![0u8; count * element_size];

        let buffer = Buffer::new("transform_storage")
            .create_storage_buffer(&data, Some(wgpu::BufferUsages::COPY_DST), ctx.renderer.device());

        let storage_buffer = buffer.storage.as_ref()
            .expect("Storage buffer should be storage");
        let bind_group = BindGroup::build(&format!("{}_storage", name))
            .add_storage_buffer(storage_buffer, visibility, true)
            .finish(ctx);

        Self { name: name.to_string(), buffer, bind_group, size: count * element_size, visibility, count }
    }

    /// Set new size for the buffer. New empty buffer will replace the old one
    pub fn resize(&mut self, count: usize, element_size: usize, ctx: &mut SystemsContext) {
        if count * element_size == self.size {
            return;
        }

        let new = Self::new(&self.name, count, element_size, ctx, self.visibility);

        self.buffer = new.buffer;
        self.bind_group = new.bind_group;
        self.size = new.size;
        self.count = new.count;
    }

    /// Update the buffer with new data
    /// Resizes the buffer if the data is larger than the current buffer size
    ///
    /// # Note
    /// Count cannot be inferred from the data, since it can be a slice of anything, 
    /// not just &[Element]
    ///
    /// # Panics
    /// Panics if the data length in bytes is not divisible by the provided count, since
    /// element_size is computed as `data_bytes.len() / count`
    pub fn update<A>(&mut self, data: &[A], count: usize, ctx: &mut SystemsContext) 
    where A: NoUninit + AnyBitPattern 
    {
        if data.is_empty() {
            return;
        }

        let data = bytemuck::cast_slice(data);

        assert_eq!(data.len() % count, 0, "Data byte length must be divisible by provided element count");

        if data.len() > self.size {
            let element_size = data.len() / count;
            self.resize(count, element_size, ctx);
        }

        let buffer = self.buffer();
        ctx.renderer.queue().write_buffer(buffer, 0, data);
    }

    /// Return the storage buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        self.buffer.storage.as_ref().expect("Storage buffer should be a storage buffer")
    }

    /// Return the bind group
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group.inner
    }

    /// Return the amount of elements in the buffer
    pub fn count(&self) -> usize {
        self.count
    }

    /// Return the size of the buffer in bytes
    pub fn size(&self) -> usize {
        self.size
    }

    /// Return the size of a single element in the buffer
    pub fn element_size(&self) -> usize {
        self.size / self.count
    }
}

pub struct TransformStorage(Storage);
pub struct LightStorage(Storage);

impl TransformStorage {
    pub fn new(n: usize, size: usize, ctx: &mut SystemsContext, visibility: wgpu::ShaderStages) -> Self {
        Self(Storage::new("transform", n, size, ctx, visibility))
    }
}

impl LightStorage {
    pub fn new(n: usize, size: usize, ctx: &mut SystemsContext, visibility: wgpu::ShaderStages) -> Self {
        Self(Storage::new("light", n, size, ctx, visibility))
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

use bytemuck::{AnyBitPattern, NoUninit};

use super::{BindGroup, Buffer};


/// Special kind of RenderAsset which is stored as a global Resource
/// Contains a buffer with instanced transform data, and the bind group
pub struct TransformStorage {
    /// Size of the buffer in bytes
    size: usize,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl TransformStorage {
    /// Create a new TransformStorage with n transforms of size bytes
    pub fn new(n: usize, size: usize, device: &wgpu::Device) -> Self {
        let data = vec![0u8; n * size];

        let buffer = Buffer::new("transform_storage")
            .create_storage_buffer(&data, Some(wgpu::BufferUsages::COPY_DST), device);

        let storage_buffer = buffer.storage.as_ref()
            .expect("TransformStorage buffer should be storage");
        let bind_group = BindGroup::build("transform_storage", device)
            .add_storage_buffer(storage_buffer, wgpu::ShaderStages::VERTEX, true)
            .finish();

        Self { buffer, bind_group, size: n * size }
    }

    /// Set new size for the buffer
    pub fn resize(&mut self, n: usize, size: usize, device: &wgpu::Device) {
        if n * size == self.size {
            return;
        }

        let new = Self::new(n, size, device);

        self.buffer = new.buffer;
        self.bind_group = new.bind_group;
    }

    /// Update the buffer with new data
    /// Resizes the buffer if the data is larger than the current buffer size
    pub fn update<A>(&mut self, data: &[A], device: &wgpu::Device, queue: &wgpu::Queue) 
    where A: NoUninit + AnyBitPattern 
    {
        let data = bytemuck::cast_slice(data);

        if data.len() > self.size {
            self.resize(data.len(), 1, device);
        }

        let buffer = self.buffer();
        queue.write_buffer(buffer, 0, data);
    }

    /// Return the storage buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        self.buffer.storage.as_ref().expect("TransformStorage buffer should be storage")
    }

    /// Return the bind group
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group.inner
    }
}

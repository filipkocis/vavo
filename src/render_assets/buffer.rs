use bytemuck::{AnyBitPattern, NoUninit};
use wgpu::util::DeviceExt;

#[derive(crate::macros::RenderAsset)]
pub struct Buffer {
    pub label: String,
    pub vertex: Option<wgpu::Buffer>,
    pub index: Option<wgpu::Buffer>,
    pub uniform: Option<wgpu::Buffer>,
    pub storage: Option<wgpu::Buffer>,
    pub num_indices: u32,
    pub num_vertices: u32,
}

impl Buffer {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            vertex: None,
            index: None,
            uniform: None,
            storage: None,
            num_indices: 0,
            num_vertices: 0,
        }
    }

    pub fn data_from_slice<A>(data: &[A]) -> &[u8]
    where A: NoUninit + AnyBitPattern {
        bytemuck::cast_slice(data)
    }

    /// Creates new vertex buffer with [wgpu::BufferUsages::VERTEX] usage. Updates `num_vertices`
    /// to the length of the data slice.
    ///
    /// # Note
    /// If the data slice is empty, `vertex` buffer will be [None].
    pub fn create_vertex_buffer<A>(self, data: &[A], usages: Option<wgpu::BufferUsages>, device: &wgpu::Device) -> Self
    where A: NoUninit + AnyBitPattern {
        let vertex_buffer = if !data.is_empty() {
            Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_vertex_buffer", self.label)),
                contents: bytemuck::cast_slice(data),
                usage: if let Some(usages) = usages { wgpu::BufferUsages::VERTEX | usages } else { wgpu::BufferUsages::VERTEX }, 
            }))
        } else {
            None
        };

        // TODO: fix num_vertices, data.len() is incorrect
        Self {
            vertex: vertex_buffer,
            num_vertices: data.len() as u32,
            ..self
        }
    }
    
    /// Creates new index buffer with [wgpu::BufferUsages::INDEX] usage. Updates `num_indices`
    /// to the length of the data slice.
    ///
    /// # Note
    /// If the data slice is empty, `index` buffer will be [None].
    pub fn create_index_buffer(self, data: &[u32], usages: Option<wgpu::BufferUsages>, device: &wgpu::Device) -> Self {
        let index_buffer = if !data.is_empty() {
            Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_index_buffer", self.label)),
                contents: bytemuck::cast_slice(data),
                usage: if let Some(usages) = usages { wgpu::BufferUsages::INDEX | usages } else { wgpu::BufferUsages::INDEX }, 
            }))
        } else {
            None
        };

        Self {
            index: index_buffer,
            num_indices: data.len() as u32,
            ..self
        }
    }

    /// Creates new uniform buffer with [wgpu::BufferUsages::UNIFORM] usage
    ///
    /// # Note
    /// If the data slice is empty, `uniform` buffer will be [None].
    pub fn create_uniform_buffer<A>(self, data: &[A], usages: Option<wgpu::BufferUsages>, device: &wgpu::Device) -> Self
    where A: NoUninit + AnyBitPattern {
        let uniform_buffer = if !data.is_empty() { 
            Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_uniform_buffer", self.label)),
                contents: bytemuck::cast_slice(data),
                usage: if let Some(usages) = usages { wgpu::BufferUsages::UNIFORM | usages } else { wgpu::BufferUsages::UNIFORM }, 
            })) 
        } else {
            None
        };

        Self {
            uniform: uniform_buffer,
            ..self
        }
    }

    /// Creates new storage buffer with [wgpu::BufferUsages::STORAGE] usage
    ///
    /// # Note
    /// If the data slice is empty, `storage` buffer will be [None].
    pub fn create_storage_buffer<A>(self, data: &[A], usages: Option<wgpu::BufferUsages>, device: &wgpu::Device) -> Self
    where A: NoUninit + AnyBitPattern {
        let storage_buffer = if !data.is_empty() {
            Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_storage_buffer", self.label)),
                contents: bytemuck::cast_slice(data),
                usage: if let Some(usages) = usages { wgpu::BufferUsages::STORAGE | usages } else { wgpu::BufferUsages::STORAGE }, 
            }))
        } else {
            None
        };

        Self {
            storage: storage_buffer,
            ..self
        }
    }
}

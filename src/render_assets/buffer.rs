use bytemuck::{AnyBitPattern, NoUninit};
use wgpu::util::DeviceExt;

pub struct Buffer {
    pub label: String,
    pub vertex: Option<wgpu::Buffer>,
    pub index: Option<wgpu::Buffer>,
    pub uniform: Option<wgpu::Buffer>,
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
            num_indices: 0,
            num_vertices: 0,
        }
    }

    pub fn data_from_slice<A>(data: &[A]) -> &[u8]
    where A: NoUninit + AnyBitPattern {
        bytemuck::cast_slice(data)
    }

    pub fn create_vertex_buffer<A>(self, data: &[A], usages: Option<wgpu::BufferUsages>, device: &wgpu::Device) -> Self
    where A: NoUninit + AnyBitPattern {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_vertex_buffer", self.label)),
            contents: bytemuck::cast_slice(data),
            usage: if let Some(usages) = usages { wgpu::BufferUsages::VERTEX | usages } else { wgpu::BufferUsages::VERTEX }, 
        });

        Self {
            vertex: Some(vertex_buffer),
            num_vertices: data.len() as u32,
            ..self
        }
    }
    
    pub fn create_index_buffer(self, data: &[u32], usages: Option<wgpu::BufferUsages>, device: &wgpu::Device) -> Self {
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_index_buffer", self.label)),
            contents: bytemuck::cast_slice(data),
            usage: if let Some(usages) = usages { wgpu::BufferUsages::INDEX | usages } else { wgpu::BufferUsages::INDEX }, 
        });

        Self {
            index: Some(index_buffer),
            num_indices: data.len() as u32,
            ..self
        }
    }

    pub fn create_uniform_buffer<A>(self, data: &[A], usages: Option<wgpu::BufferUsages>, device: &wgpu::Device) -> Self
    where A: NoUninit + AnyBitPattern {
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_uniform_buffer", self.label)),
            contents: bytemuck::cast_slice(data),
            usage: if let Some(usages) = usages { wgpu::BufferUsages::UNIFORM | usages } else { wgpu::BufferUsages::UNIFORM }, 
        });

        Self {
            uniform: Some(uniform_buffer),
            ..self
        }
    }
}

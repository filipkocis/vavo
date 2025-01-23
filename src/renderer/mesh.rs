use std::mem;

pub use wgpu::PrimitiveTopology;
use wgpu::{VertexAttribute, VertexFormat};

use crate::{render_assets::{Buffer, RenderAsset}, renderer::palette, system::SystemsContext, world::EntityId};

use super::Color;

/// Anything that can be converted into a Mesh
pub trait Meshable {
    fn mesh(&self) -> Mesh;
}

#[derive(Debug, Default, Clone)]
pub struct Mesh {
    pub topology: PrimitiveTopology,
    pub colors: Option<Vec<Color>>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
    pub indices: Option<Vec<u32>>,
}

impl Mesh {
    pub fn new(
        topology: PrimitiveTopology, 
        colors: Option<Vec<Color>>,
        positions: Vec<[f32; 3]>,
        normals: Option<Vec<[f32; 3]>>, 
        uvs: Option<Vec<[f32; 2]>>, 
        indices: Option<Vec<u32>>
    ) -> Self {
        Self {
            topology,
            colors,
            positions,
            normals,
            uvs,
            indices,
        }
    }

    pub fn from(meshable: impl Meshable) -> Self {
        meshable.mesh()
    }

    pub(crate) const VERTEX_SIZE_IN_F32: usize = 12;
    pub(crate) const VERTEX_SIZE_IN_U8: usize = 12 * std::mem::size_of::<f32>();

    fn vertex(&self, index: usize) -> [f32; Self::VERTEX_SIZE_IN_F32] {
        let color = self.colors.as_ref().map_or(palette::TRANSPARENT, |v| v[index]);
        let pos = self.positions[index];
        let normal = self.normals.as_ref().map_or([0.0, 0.0, 0.0], |v| v[index]);
        let uv = self.uvs.as_ref().map_or([0.0, 0.0], |v| v[index]);

        [
            pos[0], pos[1], pos[2],
            color.r, color.g, color.b, color.a,
            normal[0], normal[1], normal[2],
            uv[0], uv[1],
        ]
    }

    pub(crate) fn vertex_data(&self) -> Vec<f32> {
        let mut data = Vec::new();
        for i in 0..self.positions.len() {
            data.extend(self.vertex(i));
        }
        data
    }

    pub(crate) fn index_data(&self) -> Option<&[u32]> {
        self.indices.as_deref()
    }

    /// Returns the vertex buffer layout for Mesh
    pub fn vertex_descriptor() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::VERTEX_SIZE_IN_U8 as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                // Color
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                // Normal
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
                // UV
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: mem::size_of::<[f32; 10]>() as wgpu::BufferAddress,
                    shader_location: 3,
                }
            ]
        }
    }
}

impl RenderAsset<Buffer> for Mesh {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        _: Option<&EntityId>
    ) -> Buffer {
        let device = ctx.renderer.device();

        let buffer = Buffer::new("mesh")
            .create_vertex_buffer(&self.vertex_data(), None, device);

        if let Some(indices) = self.index_data() {
            buffer.create_index_buffer(indices, None, device)
        } else {
            buffer
        }
    }
}

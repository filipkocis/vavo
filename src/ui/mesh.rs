use wgpu::VertexAttribute;
use wgpu::VertexFormat;

use crate::prelude::*;
use crate::render_assets::*;

/// Mesh for UI nodes, either 2d or 3d
#[derive(Debug)]
pub struct UiMesh {
    pub colors: Vec<Color>,
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub transform_indices: Vec<u32>,
}

impl UiMesh {
    pub fn new() -> Self {
        Self {
            colors: Vec::new(),
            positions: Vec::new(),
            indices: Vec::new(),
            transform_indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.colors.clear();
        self.positions.clear();
        self.indices.clear();
        self.transform_indices.clear();
    }

    pub fn add_rect(&mut self, x: f32, y: f32, z_layer: f32, w: f32, h: f32, color: Color, transform_index: u32) {
        let i = self.positions.len() as u32;

        self.positions.extend([
            [x, y + h, z_layer],
            [x + w, y + h, z_layer],
            [x + w, y, z_layer],
            [x, y, z_layer],
        ]);

        self.indices.extend([
            i, i + 1, i + 2,
            i + 2, i + 3, i,
        ]);

        self.transform_indices.extend([
            transform_index, transform_index, transform_index, transform_index,
        ]);

        self.colors.extend([color, color, color, color]);
    }

    pub fn vertex_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        for i in 0..self.positions.len() {
            let color = self.colors[i];
            let pos = self.positions[i];
            let transform_index = self.transform_indices[i];

            data.extend([
                color.r, color.g, color.b, color.a,
                pos[0], pos[1], pos[2],
            ].into_iter().flat_map(|f| f.to_ne_bytes()));

            data.extend(transform_index.to_ne_bytes())
        }

        data
    }

    /// Returns the vertex buffer layout for Mesh
    pub fn vertex_descriptor() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Color
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 1,
                },
                // Position
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 0,
                },
                // Transform Index
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
            ]
        }
    }
}

impl RenderAsset<Buffer> for UiMesh {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        _: Option<&EntityId>
    ) -> Buffer {
        let device = ctx.renderer.device();

        Buffer::new("ui_mesh")
            .create_vertex_buffer(&self.vertex_data(), None, device)
            .create_index_buffer(&self.indices, None, device)
    }
}

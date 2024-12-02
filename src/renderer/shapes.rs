use super::{mesh::Meshable, Mesh};

pub struct Cuboid {
    pub width: f32, 
    pub height: f32,
    pub depth: f32,
}

pub struct Cube {
    pub size: f32,
}

pub struct Sphere {
    pub radius: f32,
    pub rings: usize,
    pub sectors: usize,
}

pub struct Cylinder {
    pub radius: f32,
    pub height: f32,
    pub rings: usize,
}

pub struct Cone {
    pub radius: f32,
    pub height: f32,
    pub rings: usize,
}

pub struct Torus {
    pub radius: f32,
    pub tube_radius: f32,
    pub rings: usize,
    pub sides: usize,
}

pub struct Plane {
    pub normal: [f32; 3],
    pub width: f32,
    pub height: f32,
}

impl Cuboid {
    pub fn new(width: f32, height: f32, depth: f32) -> Self {
        Self { width, height, depth }
    }
}

impl Meshable for Cuboid {
    fn mesh(&self) -> Mesh {
        let hw = self.width / 2.0; 
        let hh = self.height / 2.0;
        let hd = self.depth / 2.0;

        let vertices = &[
            // Front
            ([-hw, -hh, hd], [0.0, 0.0, 1.0], [0.0, 0.0]),
            ([hw, -hh, hd], [0.0, 0.0, 1.0], [1.0, 0.0]),
            ([hw, hh, hd], [0.0, 0.0, 1.0], [1.0, 1.0]),
            ([-hw, hh, hd], [0.0, 0.0, 1.0], [0.0, 1.0]),
            // Back
            ([-hw, hh, -hd], [0.0, 0.0, -1.0], [1.0, 0.0]),
            ([hw, hh, -hd], [0.0, 0.0, -1.0], [0.0, 0.0]),
            ([hw, -hh, -hd], [0.0, 0.0, -1.0], [0.0, 1.0]),
            ([-hw, -hh, -hd], [0.0, 0.0, -1.0], [1.0, 1.0]),
            // Right
            ([hw, -hh, -hd], [1.0, 0.0, 0.0], [0.0, 0.0]),
            ([hw, hh, -hd], [1.0, 0.0, 0.0], [1.0, 0.0]),
            ([hw, hh, hd], [1.0, 0.0, 0.0], [1.0, 1.0]),
            ([hw, -hh, hd], [1.0, 0.0, 0.0], [0.0, 1.0]),
            // Left
            ([-hw, -hh, hd], [-1.0, 0.0, 0.0], [1.0, 0.0]),
            ([-hw, hh, hd], [-1.0, 0.0, 0.0], [0.0, 0.0]),
            ([-hw, hh, -hd], [-1.0, 0.0, 0.0], [0.0, 1.0]),
            ([-hw, -hh, -hd], [-1.0, 0.0, 0.0], [1.0, 1.0]),
            // Top
            ([hw, hh, -hd], [0.0, 1.0, 0.0], [1.0, 0.0]),
            ([-hw, hh, -hd], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([-hw, hh, hd], [0.0, 1.0, 0.0], [0.0, 1.0]),
            ([hw, hh, hd], [0.0, 1.0, 0.0], [1.0, 1.0]),
            // Bottom
            ([hw, -hh, hd], [0.0, -1.0, 0.0], [0.0, 0.0]),
            ([-hw, -hh, hd], [0.0, -1.0, 0.0], [1.0, 0.0]),
            ([-hw, -hh, -hd], [0.0, -1.0, 0.0], [1.0, 1.0]),
            ([hw, -hh, -hd], [0.0, -1.0, 0.0], [0.0, 1.0]),
        ];

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        let indices = vec![
            0, 1, 2, 2, 3, 0, // front
            4, 5, 6, 6, 7, 4, // back
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20, // bottom
        ];

        Mesh::new(
            wgpu::PrimitiveTopology::TriangleList,
            None,
            positions,
            Some(normals),
            Some(uvs),
            Some(indices),
        )
    }
}

impl Cube {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
}

impl Meshable for Cube {
    fn mesh(&self) -> Mesh {
        Cuboid {
            width: self.size,
            height: self.size,
            depth: self.size,
        }.mesh()
    }
}

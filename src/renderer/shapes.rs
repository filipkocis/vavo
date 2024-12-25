use super::{mesh::Meshable, sphere::SphereKind, Mesh};

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
    pub kind: SphereKind,
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
    pub width: f32,
    pub height: f32,
    pub face_down: bool,
}

pub struct Triangle {
    pub vertices: [[f32; 3]; 3],
} 

impl Triangle {
    pub fn equilateral(base: f32) -> Self {
        let height = (3.0_f32.sqrt() / 2.0) * base;
        let half_height = height / 2.0;
        let half_base = base / 2.0;

        Self {
            vertices: [
                [0.0, half_height, 0.0],
                [-half_base, -half_height, 0.0],
                [half_base, -half_height, 0.0],
            ]
        }
    }
}

impl Meshable for Triangle {
    fn mesh(&self) -> Mesh {
        let positions = self.vertices.iter().map(|v| *v).collect(); 
        let normals = vec![[0.0, 0.0, 1.0]; 3];
        let uvs = vec![
            [0.5, 1.0],
            [0.0, 0.0],
            [1.0, 0.0],
        ];
        let indices = vec![0, 1, 2];
        
        Mesh::new(
            wgpu::PrimitiveTopology::TriangleList,
            None,
            positions,
            Some(normals),
            Some(uvs),
            Some(indices)
        )
    }
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

impl Sphere {
    /// Generate a new icosphere with 3 subdivisions
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            kind: SphereKind::Icosphere(3),
        }
    }

    pub fn ico(radius: f32, subdivisions: u32) -> Self {
        Self {
            radius,
            kind: SphereKind::Icosphere(subdivisions),
        }
    }

    pub fn uv(radius: f32, rings: u32, sectors: u32) -> Self {
        Self {
            radius,
            kind: SphereKind::UVSphere(rings, sectors), 
        }
    }
}

impl Meshable for Sphere {
    fn mesh(&self) -> Mesh {
        let (positions, uvs, normals, indices) = match self.kind {
            SphereKind::Icosphere(subdivisions) => {
                Self::generate_icosphere(self.radius, subdivisions)
            },
            SphereKind::UVSphere(rings, sectors) => {
                Self::generate_uv_sphere(self.radius, rings, sectors)
            }
        };

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

impl Plane {
    pub fn new(width: f32, height: f32, face_down: bool) -> Self {
        Self {
            width,
            height,
            face_down,
        } 
    }
}

impl Meshable for Plane {
    fn mesh(&self) -> Mesh {
        let hw = self.width / 2.0;
        let hh = self.height / 2.0;

        let vertices = &[
            ([-hw, 0.0, hh], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([hw, 0.0, hh], [0.0, 1.0, 0.0], [1.0, 0.0]),
            ([hw, 0.0, -hh], [0.0, 1.0, 0.0], [1.0, 1.0]),
            ([-hw, 0.0, -hh], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ];

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        let indices = vec![0, 1, 2, 2, 3, 0];

        Mesh::new(
            wgpu::PrimitiveTopology::TriangleList,
            None,
            positions,
            Some(normals),
            Some(uvs),
            Some(indices)
        )
    }
}

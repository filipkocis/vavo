use crate::math::shapes::{Cube, Cuboid, Plane, Sphere, SphereKind, Triangle};

use super::{Mesh, Meshable};

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

impl Meshable for Cube {
    fn mesh(&self) -> Mesh {
        Cuboid {
            width: self.size,
            height: self.size,
            depth: self.size,
        }
        .mesh()
    }
}

impl Meshable for Sphere {
    fn mesh(&self) -> Mesh {
        let (positions, uvs, normals, indices) = match self.kind {
            SphereKind::Icosphere(subdivisions) => {
                Self::generate_icosphere(self.radius, subdivisions)
            }
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
            Some(indices),
        )
    }
}

impl Meshable for Triangle {
    fn mesh(&self) -> Mesh {
        let positions = self.vertices.to_vec();
        let normals = vec![[0.0, 0.0, 1.0]; 3];
        let uvs = vec![[0.5, 1.0], [0.0, 0.0], [1.0, 0.0]];
        let indices = vec![0, 1, 2];

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

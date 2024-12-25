use std::collections::HashMap;

use super::shapes::Sphere;

pub enum SphereKind {
    /// Defines a icosphere with N subdivisions
    Icosphere(u32),
    /// UV sphere with N rings (latitude) and M sides (longitude)
    UVSphere(u32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct EdgeKey(usize, usize);

impl EdgeKey {
    fn new(a: usize, b: usize) -> Self {
        if a < b {
            EdgeKey(a, b)
        } else {
            EdgeKey(b, a)
        }
    }
}

impl Sphere {
    pub(super) fn generate_uv_sphere(_radius: f32, _rings: u32, _sectors: u32) -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<[f32; 3]>, Vec<u32>) {
        unimplemented!("UV sphere generation");
    }

    pub(super) fn generate_icosphere(radius: f32, subdivisions: u32) -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<[f32; 3]>, Vec<u32>) {
        let t = (1.0 + 5.0f32.sqrt()) / 2.0;

        let mut positions = vec![
            [-1.0,  t,  0.0],
            [ 1.0,  t,  0.0],
            [-1.0, -t,  0.0],
            [ 1.0, -t,  0.0],
            [ 0.0, -1.0,  t],
            [ 0.0,  1.0,  t],
            [ 0.0, -1.0, -t],
            [ 0.0,  1.0, -t],
            [  t,  0.0, -1.0],
            [  t,  0.0,  1.0],
            [ -t,  0.0, -1.0],
            [ -t,  0.0,  1.0],
        ];

        let mut indices = vec![
            [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
            [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
            [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
            [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
        ];

        for v in &mut positions {
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            v[0] /= len;
            v[1] /= len;
            v[2] /= len;
        }

        let mut edge_map = HashMap::new();
        for _ in 0..subdivisions {
            let mut new_indices = Vec::new();

            for &[v1, v2, v3] in &indices {
                let a = get_middle_point(v1, v2, &mut positions, &mut edge_map);
                let b = get_middle_point(v2, v3, &mut positions, &mut edge_map);
                let c = get_middle_point(v3, v1, &mut positions, &mut edge_map);

                new_indices.push([v1, a, c]);
                new_indices.push([v2, b, a]);
                new_indices.push([v3, c, b]);
                new_indices.push([a, b, c]);
            }

            indices = new_indices;
        }

        for v in &mut positions {
            v[0] *= radius;
            v[1] *= radius;
            v[2] *= radius;
        }

        let mut normals = Vec::new();
        let mut uvs = Vec::new();

        for v in &positions {
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            normals.push([v[0] / len, v[1] / len, v[2] / len]);

            let u = 0.5 + (v[2].atan2(v[0]) / (2.0 * std::f32::consts::PI));
            let v = 0.5 - (v[1].asin() / std::f32::consts::PI);
            uvs.push([u, v]);
        }

        let indices: Vec<u32> = indices.into_iter().flat_map(|tri| [tri[0] as u32, tri[1] as u32, tri[2] as u32]).collect();

        (positions, uvs, normals, indices)
    }
}

fn get_middle_point(v1: usize, v2: usize, positions: &mut Vec<[f32; 3]>, edge_map: &mut HashMap<EdgeKey, usize>) -> usize {
    let edge = EdgeKey::new(v1, v2);

    if let Some(&index) = edge_map.get(&edge) {
        return index;
    }

    let midpoint = [
        (positions[v1][0] + positions[v2][0]) / 2.0,
        (positions[v1][1] + positions[v2][1]) / 2.0,
        (positions[v1][2] + positions[v2][2]) / 2.0,
    ];

    let len = (midpoint[0] * midpoint[0] + midpoint[1] * midpoint[1] + midpoint[2] * midpoint[2]).sqrt();
    let midpoint = [midpoint[0] / len, midpoint[1] / len, midpoint[2] / len];

    let index = positions.len();
    positions.push(midpoint);
    edge_map.insert(edge, index);

    index
}

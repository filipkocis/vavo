use super::sphere::SphereKind;

pub struct Cuboid {
    pub width: f32, 
    pub height: f32,
    pub depth: f32,
}

impl Cuboid {
    pub fn new(width: f32, height: f32, depth: f32) -> Self {
        Self { width, height, depth }
    }
}

pub struct Cube {
    pub size: f32,
}

impl Cube {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
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

pub struct Plane {
    pub width: f32,
    pub height: f32,
    pub face_down: bool,
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

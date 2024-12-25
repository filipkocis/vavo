use glam::Vec3;

pub enum CubeFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl CubeFace {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => CubeFace::PosX,
            1 => CubeFace::NegX,
            2 => CubeFace::PosY,
            3 => CubeFace::NegY,
            4 => CubeFace::PosZ,
            5 => CubeFace::NegZ,
            _ => panic!("Invalid cube map face index"),
        }
    }

    pub fn index(&self) -> usize {
        match self {
            CubeFace::PosX => 0,
            CubeFace::NegX => 1,
            CubeFace::PosY => 2,
            CubeFace::NegY => 3,
            CubeFace::PosZ => 4,
            CubeFace::NegZ => 5,
        }
    }

    pub fn vector(&self) -> Vec3 {
        match self {
            CubeFace::PosX => Vec3::X,
            CubeFace::NegX => Vec3::NEG_X,
            CubeFace::PosY => Vec3::Y,
            CubeFace::NegY => Vec3::NEG_Y,
            CubeFace::PosZ => Vec3::Z,
            CubeFace::NegZ => Vec3::NEG_Z,
        }
    }
}

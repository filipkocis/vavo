use crate::entities::Entities;

pub struct World {
    pub entities: Entities,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
        }
    }
}

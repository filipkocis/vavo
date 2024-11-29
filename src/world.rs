use crate::entities::Entities;

pub struct World {
    pub entities: Entities,
    pub resources: Resources,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
        }
    }
}

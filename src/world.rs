use crate::{entities::Entities, resources::Resources};

pub struct World {
    pub entities: Entities,
    pub resources: Resources,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
            resources: Resources::new(),
        }
    }
}

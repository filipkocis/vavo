use crate::resources::Resources;

use super::entities::Entities;

pub struct World {
    pub entities: Entities,
    pub resources: Resources,
}

impl World {
    pub fn new() -> Self {
        let mut resources = Resources::new();
        resources.insert_default_resources();

        Self {
            entities: Entities::new(),
            resources,
        }
    }
}

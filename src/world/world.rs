use super::{entities::Entities, resources::Resources};

pub(crate) struct World {
    pub entities: Entities,
    pub resources: Resources,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
            resources: Resources::with_default_resources(),
        }
    }
}

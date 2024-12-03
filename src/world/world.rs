use crate::resources::Resources;

use super::{entities::Entities};

pub(crate) struct World {
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

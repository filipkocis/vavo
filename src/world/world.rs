use crate::resources::{Resources, Time};

use super::entities::Entities;

pub(crate) struct World {
    pub entities: Entities,
    pub resources: Resources,
}

impl World {
    pub fn new() -> Self {
        let mut resources = Resources::new();
        resources.insert_default_resources();

        let time = resources.get::<Time>().unwrap();
        let entities = Entities::new(time.tick_raw());

        Self {
            entities,
            resources,
        }
    }
}

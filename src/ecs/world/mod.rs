use crate::query::Query;

use super::entities::Entities;
use super::resources::Resources;
use super::tick::Tick;

pub struct World {
    pub entities: Entities,
    pub resources: Resources,
    /// Current world tick
    pub tick: Tick,
}

impl World {
    pub fn new() -> Self {
        let resources = Resources::new();

        let mut world = Self {
            entities: Entities::new(),
            resources,
            tick: Tick::default(),
        };

        // Initialize entities
        // world.entities.initialize_tick(&world.tick);

        // Initialize resources
        world.resources.initialize_tick(&world.tick);
        world.resources.insert_default_resources();

        world
    }

    /// Creates new world query
    pub fn query<T>(&mut self) -> Query<T> {
        Query::new(&mut self.entities)
    }

    /// Creates new world query with filters
    pub fn query_filtered<T, F>(&mut self) -> Query<T, F> {
        Query::new(&mut self.entities)
    }
}

use crate::query::Query;

use super::entities::components::ComponentsRegistry;
use super::entities::Entities;
use super::resources::Resources;
use super::tick::Tick;

pub struct World {
    pub entities: Entities,
    pub resources: Resources,
    /// Current world tick
    pub tick: Box<Tick>,
    /// Component types metadata registry
    pub registry: ComponentsRegistry,
}

impl World {
    pub fn new() -> Self {
        let tick = Box::new(Tick::default());

        let mut world = Self {
            entities: Entities::new(),
            resources: Resources::new(),
            tick,
            registry: ComponentsRegistry::new(),
        };

        // Initialize entities
        world.entities.initialize_tick(world.tick.as_ref());

        // Initialize resources
        world.resources.initialize_tick(world.tick.as_ref());
        world.resources.insert_default_resources();

        world
    }

    /// Update function for the world, updates world tick and resources
    pub(crate) fn update(&mut self) {
        self.tick.increment();
        self.resources.update();
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

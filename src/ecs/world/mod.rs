use crate::prelude::EntityId;
use crate::query::Query;
use crate::system::Commands;
use crate::system::commands::CommandQueue;

use super::entities::Entities;
use super::entities::components::ComponentsRegistry;
use super::resources::Resources;
use super::tick::Tick;

pub struct World {
    pub entities: Entities,
    pub resources: Resources,
    /// Current world tick
    pub tick: Box<Tick>,
    /// Component types metadata registry
    pub registry: ComponentsRegistry,

    command_queue: CommandQueue,
}

impl Default for World {
    fn default() -> Self {
        let tick = Box::new(Tick::default());

        let mut world = Self {
            entities: Entities::new(),
            resources: Resources::new(),
            tick,
            registry: ComponentsRegistry::new(),
            command_queue: CommandQueue::new(),
        };

        // Initialize entities
        world.entities.initialize(
            world.tick.as_ref(),
            world.registry.get_or_register::<EntityId>(),
        );

        // Initialize resources
        world.resources.initialize_tick(world.tick.as_ref());
        world.resources.insert_default_resources();

        world
    }
}

impl World {
    /// Creates a new empty world
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update function for the world, updates resources
    #[inline]
    pub(crate) fn update(&mut self) {
        self.resources.update();
    }

    /// Creates new world query
    /// It is without a system execution context
    #[inline]
    pub fn query<T>(&mut self) -> Query<T> {
        Query::new(&mut self.entities, *self.tick)
    }

    /// Creates new world query with filters
    /// It is without a system execution context
    #[inline]
    pub fn query_filtered<T, F>(&mut self) -> Query<T, F> {
        Query::new(&mut self.entities, *self.tick)
    }

    /// Returns new commands instance with the world's command queue
    #[inline]
    pub fn commands<'a, 'b>(&'a mut self) -> Commands<'a, 'b>
    where
        'a: 'b,
        'b: 'a,
    {
        Commands::new(&mut self.entities.tracking, &mut self.command_queue)
    }

    /// Flushes all queued commands to the world
    #[inline]
    pub(crate) fn flush_commands(&mut self) {
        let world = unsafe { &mut *(self as *mut _) };
        self.command_queue.apply(world)
    }
}

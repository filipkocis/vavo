use crate::app::App;
use crate::prelude::EntityId;
use crate::query::Query;
use crate::renderer::newtype::{RenderCommandQueue, RenderQueue};
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

    /// The parent app that contains this world, if any.
    /// Needed for systems that require access to fields only available in App.
    /// TODO: This will be removed in the future once those fields are available through other
    /// means, such as resource wrappers.
    pub(crate) parent_app: *mut App,

    /// Main command queue for the world
    pub(crate) command_queue: CommandQueue,
    pub(crate) render_command_queue: RenderCommandQueue,
}

// Safety: World needs to be Send and Sync so we can use it in schedulers that run systems in
// parallel. Scheduler manages safe access to the world.
unsafe impl Send for World {}
unsafe impl Sync for World {}

impl Default for World {
    fn default() -> Self {
        let tick = Box::new(Tick::default());

        let mut world = Self {
            entities: Entities::new(),
            resources: Resources::new(),
            tick,
            registry: ComponentsRegistry::new(),
            parent_app: std::ptr::null_mut(),
            command_queue: CommandQueue::new(),
            render_command_queue: RenderCommandQueue::new(),
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

    /// Returns a mutable reference to the parent app.
    ///
    /// # Safety
    /// Will panic if the world was not created within an app.
    /// Completely unsafe as the pointer allows for mutable aliasing.
    #[inline]
    pub(crate) unsafe fn parent_app(&mut self) -> &mut App {
        if self.parent_app.is_null() {
            panic!("World was not created within an App, parent_app is null.");
        } else {
            unsafe { &mut *self.parent_app }
        }
    }

    /// Flushes all queued commands to the world
    #[inline]
    pub(crate) fn flush_commands(&mut self) {
        let world = unsafe { &mut *(self as *mut _) };
        self.command_queue.apply(world)
    }

    /// Flushes all queued render commands to the world
    #[inline]
    pub(crate) fn flush_render_commands(&mut self) {
        let queue = self.resources.get_mut::<RenderQueue>();
        queue.submit(self.render_command_queue.drain());
    }

    /// Reborrows the world as a mutable reference with a different lifetime.
    ///
    /// # Safety
    /// This is unsafe because it can lead to aliasing mutable references if used improperly.
    #[inline]
    pub(crate) unsafe fn reborrow<'a, 'b>(&'a mut self) -> &'b mut World {
        unsafe { &mut *(self as *mut World) }
    }
}

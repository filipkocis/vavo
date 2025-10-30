use std::any::{TypeId, type_name};

use winit::dpi::PhysicalSize;
use winit::event::ElementState;
use winit::keyboard::PhysicalKey;

use crate::core::graph::RenderGraph;
use crate::prelude::{FixedTime, Resource};
use crate::reflect::{Reflect, registry::ReflectTypeRegistry};
use crate::system::commands::CommandQueue;
use crate::system::{Commands, IntoSystem, SystemHandler, SystemStage, SystemsContext};
use crate::window::{AppHandler, AppState, RenderContext, Renderer};

use crate::ecs::state::{NextState, State, States, systems::apply_state_transition};
use crate::ecs::world::World;
use crate::event::{
    Events,
    events::{KeyboardInput, MouseInput},
};

use super::Plugin;
use super::input::{Input, KeyCode, MouseButton};

pub struct App {
    system_handler: SystemHandler,
    render_graph: RenderGraph,

    pub world: World,
    events: Events,

    known_states: Vec<TypeId>,
    pub type_registry: ReflectTypeRegistry,
}

impl App {
    /// Create a new App
    pub fn build() -> Self {
        Self {
            system_handler: SystemHandler::new(),
            render_graph: RenderGraph::new(),
            world: World::new(),
            events: Events::new(),
            known_states: Vec::new(),
            type_registry: ReflectTypeRegistry::new(),
        }
    }

    fn add_state_internal<S: States>(&mut self, state: State<S>) {
        let state_type = TypeId::of::<S>();
        if !self.known_states.contains(&state_type) {
            self.known_states.push(state_type);

            self.world.resources.insert(state);
            self.world.resources.insert(NextState::<S>::new());

            self.register_system(apply_state_transition::<S>, SystemStage::FrameEnd);
        } else {
            panic!("State 'State<{}>' already registered", type_name::<S>());
        }
    }

    /// Add new state with a default value to the app
    pub fn register_state<S: States + Default>(&mut self) -> &mut Self {
        self.add_state_internal(State::<S>::new());
        self
    }

    /// Add new state with a specified value to the app
    pub fn add_state<S: States>(&mut self, state: S) -> &mut Self {
        self.add_state_internal(State(state));
        self
    }

    /// Register new reflectable type to the app, enabling transformation of &dyn Any components
    /// into &dyn Reflect via the [`type registry`](ReflectTypeRegistry).
    pub fn register_type<R: Reflect>(&mut self) -> &mut Self {
        self.type_registry.register::<R>();
        self
    }

    /// Add new resource with a default value to the app if it doesn't exist
    pub fn init_resource<R: Resource + Default>(&mut self) -> &mut Self {
        if !self.world.resources.contains::<R>() {
            self.world.resources.insert(R::default());
        }
        self
    }

    /// Add new resource with a specified value to the app
    pub fn set_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.resources.insert(resource);
        self
    }

    /// Write event T to the event queue
    pub fn create_event<T: 'static>(&mut self, event: T) {
        self.events.write(event);
    }

    /// Add a system to the startup stage
    pub fn add_startup_system<T, F>(&mut self, system: impl IntoSystem<T, F>) -> &mut Self {
        let system = system.system();
        self.system_handler
            .register_system(system, SystemStage::Startup);
        self
    }

    /// Add a system to the update stage
    pub fn add_system<T, F>(&mut self, system: impl IntoSystem<T, F>) -> &mut Self {
        let system = system.system();
        self.system_handler
            .register_system(system, SystemStage::Update);
        self
    }

    /// Register a system to a specific stage
    pub fn register_system<T, F>(
        &mut self,
        system: impl IntoSystem<T, F>,
        stage: SystemStage,
    ) -> &mut Self {
        let system = system.system();
        self.system_handler.register_system(system, stage);
        self
    }

    /// Add a plugin to the app
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    fn run_systems(&mut self, stage: SystemStage, renderer: Renderer, queue: &mut CommandQueue) {
        let self_ptr = self as *mut App;
        let systems = self.system_handler.get_systems(stage);
        if systems.is_empty() {
            return;
        }

        let tracking = unsafe {
            // Safety: Tracking is not accessible from system contexts
            &mut (*(&mut self.world.entities.tracking as *mut _))
        };
        // let mut queue = CommandQueue::default();
        let commands = Commands::new(tracking, queue);

        let mut ctx = SystemsContext::new(
            commands,
            &mut self.world.resources,
            &mut self.events,
            renderer,
            self_ptr,
            &mut self.render_graph,
        );

        let iterations = if stage.has_fixed_time() {
            let mut fixed_time = ctx.resources.get_mut::<FixedTime>();
            fixed_time.iter()
        } else {
            1
        };

        for _ in 0..iterations {
            for system in systems.iter_mut() {
                system.run(&mut ctx, &mut self.world.entities);
            }
        }

        ctx.commands.apply(&mut self.world);

    /// Get a mutable reference to the render graph. Use [Self::reborrow] in combination with this.
    ///
    /// # Safety
    /// The render graph should only be accessed from startup systems to edit nodes in the grpah.
    #[inline]
    pub unsafe fn render_graph(&mut self) -> &mut RenderGraph {
        &mut self.render_graph
    }

    /// Reborrows the app as a mutable reference with a different lifetime.
    ///
    /// # Safety
    /// This is unsafe because it can lead to aliasing mutable references if used improperly.
    #[inline]
    pub unsafe fn reborrow<'a, 'b>(&'a mut self) -> &'b mut App {
        unsafe { &mut *(self as *mut App) }
    }

    fn execute_render_graph(&mut self, renderer: Renderer, queue: &mut CommandQueue) {
        let tracking = unsafe {
            // Safety: Tracking is not accessible from system contexts
            &mut (*(&mut self.world.entities.tracking as *mut _))
        };
        let commands = Commands::new(tracking, queue);

        let self_ptr = self as *mut App;
        let mut ctx = SystemsContext::new(
            commands,
            &mut self.world.resources,
            &mut self.events,
            renderer,
            self_ptr,
            &mut self.render_graph,
        );

        self.render_graph
            .execute(&mut ctx, &mut self.world.entities);

        ctx.commands.apply(&mut self.world);
    }

    /// Initialize the app and run all startup systems
    pub(crate) fn startup(&mut self, state: &mut AppState) {
        let mut context = RenderContext::new_update_context(state);
        let mut queue = CommandQueue::default();

        self.run_systems(SystemStage::PreStartup, context.as_renderer(), &mut queue);
        self.run_systems(SystemStage::Startup, context.as_renderer(), &mut queue);

        self.system_handler.clear_systems(SystemStage::PreStartup);
        self.system_handler.clear_systems(SystemStage::Startup);
    }

    /// Update the app and run all update systems
    pub(crate) fn update(&mut self, state: &mut AppState) {
        let mut context = RenderContext::new_update_context(state);
        let mut queue = CommandQueue::default();

        self.world.update();

        self.run_systems(SystemStage::First, context.as_renderer(), &mut queue);
        self.run_systems(SystemStage::PreUpdate, context.as_renderer(), &mut queue);
        self.run_systems(SystemStage::FixedUpdate, context.as_renderer(), &mut queue);
        self.run_systems(SystemStage::Update, context.as_renderer(), &mut queue);
        self.run_systems(SystemStage::PostUpdate, context.as_renderer(), &mut queue);
        self.run_systems(SystemStage::Last, context.as_renderer(), &mut queue);
    }

    /// Resize the app
    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        self.render_graph.resize(size);
    }

    /// Run the app
    pub fn run(&mut self) {
        let (event_loop, mut app) = AppHandler::init(self);
        event_loop.run_app(&mut app).unwrap();

        // let systems = &self.systems;
        // let eq = systems[0].func_ptr == systems[1].func_ptr;
        // println!("eq: {} | {} == {}", eq, systems[0].name, systems[1].name);
    }

    /// Render the app and run all render systems, and execute the render graph
    pub(crate) fn render(&mut self, state: &mut AppState) -> Result<(), wgpu::SurfaceError> {
        let mut context = RenderContext::new_render_context(state)?;
        let mut queue = CommandQueue::default();

        self.run_systems(SystemStage::PreRender, context.as_renderer(), &mut queue);
        self.run_systems(SystemStage::Render, context.as_renderer(), &mut queue);
        self.execute_render_graph(context.as_renderer(), &mut queue);
        self.run_systems(SystemStage::PostRender, context.as_renderer(), &mut queue);
        self.run_systems(SystemStage::FrameEnd, context.as_renderer(), &mut queue);

        self.events.apply();

        Ok(())
    }

    /// Handle keyboard input
    pub(crate) fn handle_keyboard_input(&mut self, event: winit::event::KeyEvent) {
        let code = match event.physical_key {
            PhysicalKey::Code(code) => code,
            _ => return,
        };

        let event = KeyboardInput {
            code,
            state: event.state,
        };

        let mut input = self.world.resources.get_mut::<Input<KeyCode>>();
        if event.state == ElementState::Pressed {
            input.press(event.code);
        } else {
            input.release(event.code);
        }

        self.events.write_immediately(event);
    }

    /// Handle mouse input
    pub(crate) fn handle_mouse_input(
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) {
        let event = MouseInput { button, state };

        let mut input = self.world.resources.get_mut::<Input<MouseButton>>();
        if event.state == ElementState::Pressed {
            input.press(event.button);
        } else {
            input.release(event.button);
        }

        self.events.write_immediately(event);
    }
}

use std::any::{type_name, TypeId};

use winit::dpi::PhysicalSize;
use winit::keyboard::PhysicalKey;

use crate::core::graph::RenderGraph;
use crate::prelude::FixedTime;
use crate::state::systems::apply_state_transition;
use crate::state::{NextState, State, States};
use crate::system::{Commands, IntoSystem, SystemHandler, SystemStage, SystemsContext};
use crate::window::{AppHandler, AppState, RenderContext, Renderer};
use crate::world::World;

use super::events::{KeyboardInput, MouseInput};
use super::input::{Input, KeyCode, MouseButton};
use super::{Events, Plugin};

pub struct App {
    system_handler: SystemHandler,
    render_graph: RenderGraph,

    pub world: World,
    events: Events,

    known_states: Vec<TypeId>,
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

    /// Write event T to the event queue
    pub fn create_event<T: 'static>(&mut self, event: T) {
        self.events.write(event);
    }

    /// Add a system to the startup stage
    pub fn add_startup_system<T, F>(&mut self, system: impl IntoSystem<T, F>) -> &mut Self {
        let system = system.system();
        self.system_handler.register_system(system, SystemStage::Startup);
        self
    }

    /// Add a system to the update stage
    pub fn add_system<T, F>(&mut self, system: impl IntoSystem<T, F>) -> &mut Self {
        let system = system.system();
        self.system_handler.register_system(system, SystemStage::Update);
        self
    }

    /// Register a system to a specific stage
    pub fn register_system<T, F>(&mut self, system: impl IntoSystem<T, F>, stage: SystemStage) -> &mut Self {
        let system = system.system();
        self.system_handler.register_system(system, stage);
        self
    }

    /// Add a plugin to the app
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    fn run_systems(&mut self, stage: SystemStage, renderer: Renderer) {
        let systems = self.system_handler.get_systems(&stage);
        if systems.is_empty() {
            return;
        }

        let commands = Commands::build(&self.world);
        let world_ptr = &mut self.world as *mut World;
        let mut ctx = SystemsContext::new(commands, &mut self.world.resources, &mut self.events, renderer, world_ptr, &mut self.render_graph);

        let iterations = if stage.has_fixed_time() {
            let mut fixed_time = ctx.resources.get_mut::<FixedTime>()
                .expect("FixedTime resource not found");
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
    }

    fn execute_render_graph(&mut self, renderer: Renderer) {
        let commands = Commands::build(&self.world);
        let world_ptr = &mut self.world as *mut World;
        let mut ctx = SystemsContext::new(commands, &mut self.world.resources, &mut self.events, renderer, world_ptr, &mut self.render_graph);

        self.render_graph.execute(&mut ctx, &mut self.world.entities);

        ctx.commands.apply(&mut self.world);
    }

    /// Initialize the app and run all startup systems
    pub(crate) fn startup(&mut self, state: &mut AppState) {
        let mut context = RenderContext::new_update_context(state);

        self.run_systems(SystemStage::PreStartup, context.as_renderer());
        self.run_systems(SystemStage::Startup, context.as_renderer());
    }

    /// Update the app and run all update systems
    pub(crate) fn update(&mut self, state: &mut AppState) {
        let mut context = RenderContext::new_update_context(state);

        self.world.resources.update();

        self.run_systems(SystemStage::First, context.as_renderer());
        self.run_systems(SystemStage::PreUpdate, context.as_renderer());
        self.run_systems(SystemStage::FixedUpdate, context.as_renderer());
        self.run_systems(SystemStage::Update, context.as_renderer());
        self.run_systems(SystemStage::PostUpdate, context.as_renderer());
        self.run_systems(SystemStage::Last, context.as_renderer());
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

        self.run_systems(SystemStage::PreRender, context.as_renderer());
        self.run_systems(SystemStage::Render, context.as_renderer());
        self.execute_render_graph(context.as_renderer());
        self.run_systems(SystemStage::PostRender, context.as_renderer());
        self.run_systems(SystemStage::FrameEnd, context.as_renderer());

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
            code: code,
            state: event.state,
        };

        let mut input = self.world.resources.get_mut::<Input<KeyCode>>().expect("Input<KeyCode> resource not found");
        if event.state == winit::event::ElementState::Pressed {
            input.press(event.code);
        } else {
            input.release(event.code);
        }

        self.events.write_immediately(event);
    }

    /// Handle mouse input
    pub(crate) fn handle_mouse_input(&mut self, state: winit::event::ElementState, button: winit::event::MouseButton) {
        let event = MouseInput {
            button, state
        };

        let mut input = self.world.resources.get_mut::<Input<MouseButton>>().expect("Input<MouseButton> resource not found");
        if event.state == winit::event::ElementState::Pressed {
            input.press(event.button);
        } else {
            input.release(event.button);
        }

        self.events.write_immediately(event);
    }
}

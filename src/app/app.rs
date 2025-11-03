use std::any::{TypeId, type_name};

use winit::dpi::PhysicalSize;
use winit::event::ElementState;
use winit::keyboard::PhysicalKey;

use crate::core::graph::RenderGraph;
use crate::ecs::state::systems::register_state_events;
use crate::event::{Event, apply_events};
use crate::prelude::{FixedTime, Resource};
use crate::reflect::{Reflect, registry::ReflectTypeRegistry};
use crate::renderer::newtype::{
    RenderSurface, RenderSurfaceConfiguration, RenderSurfaceTexture, RenderSurfaceTextureView,
};
use crate::system::{IntoSchedulerLocation, IntoSystem, PhaseLabel, Scheduler, SystemParam, phase};
use crate::window::AppHandler;

use crate::ecs::state::{NextState, State, States, systems::apply_state_transition};
use crate::ecs::world::World;
use crate::event::{Events, KeyboardInput, MouseInput};

use super::Plugin;
use super::input::{Input, KeyCode, MouseButton};

pub struct App {
    scheduler: Scheduler,
    render_graph: RenderGraph,

    pub world: World,

    known_states: Vec<TypeId>,
    known_events: Vec<TypeId>,
    pub type_registry: ReflectTypeRegistry,
}

impl App {
    /// Create a new App
    pub fn build() -> Self {
        Self {
            scheduler: Scheduler::new(),
            render_graph: RenderGraph::new(),
            world: World::new(),
            known_states: Vec::new(),
            known_events: Vec::new(),
            type_registry: ReflectTypeRegistry::new(),
        }
    }

    fn add_state_internal<S: States>(&mut self, state: State<S>) {
        let state_type = TypeId::of::<S>();
        if !self.known_states.contains(&state_type) {
            self.known_states.push(state_type);

            self.world.resources.insert(state);
            self.world.resources.insert(NextState::<S>::new());

            self.register_system(register_state_events::<S>, phase::Startup);
            self.register_system(apply_state_transition::<S>, phase::FrameEnd);
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

    /// Register new event type to the app
    pub fn register_event<E: Event>(&mut self) -> &mut Self {
        let event_type = TypeId::of::<E>();
        if !self.known_events.contains(&event_type) {
            self.known_events.push(event_type);

            self.world.resources.insert(Events::<E>::new());

            self.register_system(apply_events::<E>, phase::First);
        } else {
            panic!("Event '{}' already registered", type_name::<E>());
        }
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
    #[inline]
    pub fn create_event<E: Event>(&mut self, event: E) {
        self.world.resources.get_mut::<Events<E>>().write(event);
    }

    /// Add a system to the startup phase
    pub fn add_startup_system<Params: SystemParam>(
        &mut self,
        system: impl IntoSystem<Params>,
    ) -> &mut Self {
        self.scheduler.add_system(system.build(), phase::Startup);
        self
    }

    /// Add a system to the update phase
    pub fn add_system<Params: SystemParam>(
        &mut self,
        system: impl IntoSystem<Params>,
    ) -> &mut Self {
        self.scheduler.add_system(system.build(), phase::Update);
        self
    }

    /// Register a system to a specific phase and layer location
    pub fn register_system<Params: SystemParam>(
        &mut self,
        system: impl IntoSystem<Params>,
        location: impl IntoSchedulerLocation,
    ) -> &mut Self {
        self.scheduler.add_system(system.build(), location);
        self
    }

    /// Add a plugin to the app
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

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
    pub unsafe fn reborrow<'a>(&mut self) -> &'a mut App {
        unsafe { &mut *(self as *mut App) }
    }

    /// Initialize the app
    fn initialize(&mut self) {
        self.world.parent_app = self as *mut App;

        // tepmorary system to execute render graph
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        struct RenderGraphPhase;
        impl PhaseLabel for RenderGraphPhase {}

        self.scheduler
            .pending_changes
            .phase_add(RenderGraphPhase)
            .phase_after(RenderGraphPhase, phase::Render)
            .phase_before(RenderGraphPhase, phase::PostRender);

        let execute_render_graph_system = |world: &mut World, render_graph: &mut RenderGraph| {
            render_graph.execute(world);
            world.flush_commands();
        };

        self.scheduler
            .add_system(execute_render_graph_system.build(), RenderGraphPhase);
    }

    /// Initialize the app and run startup phases
    pub(crate) fn startup(&mut self) {
        self.initialize();

        self.scheduler
            .execute_phase(&mut self.world, phase::PreStartup);
        self.scheduler
            .execute_phase(&mut self.world, phase::Startup);
    }

    /// Resize the app
    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        self.render_graph.resize(size);
    }

    /// Run the app event loop
    pub fn run(&mut self) {
        let (event_loop, mut app) = AppHandler::init(self);
        event_loop.run_app(&mut app).unwrap();
    }

    /// Execute the system scheduler for one frame
    #[inline]
    pub fn execute_scheduler(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Update
        self.world.update();

        // This should happen before rendering, for now we do it here
        self.prepare_surface()?;

        // Run all systems
        self.scheduler.execute_pipeline(&mut self.world);

        // Present surface
        self.finish_surface();
        Ok(())
    }

    /// Prepare the surface resources for rendering (called at the beginning of render)
    #[inline]
    fn prepare_surface(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface_config = self.world.resources.get::<RenderSurfaceConfiguration>();
        let surface = self.world.resources.get::<RenderSurface>();

        let surface_texture = surface.get_current_texture()?;
        let surface_texture_view =
            surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Surface Texture View"),
                    format: Some(surface_config.format),
                    ..Default::default()
                });

        let surface_texture = RenderSurfaceTexture::new(surface_texture);
        let surface_texture_view = RenderSurfaceTextureView::new(surface_texture_view);

        self.world.resources.insert(surface_texture);
        self.world.resources.insert(surface_texture_view);

        Ok(())
    }

    /// Finish rendering to the surface (called at the end of render)
    #[inline]
    fn finish_surface(&mut self) {
        self.world.flush_render_commands();

        let surface_texture = self
            .world
            .resources
            .remove::<RenderSurfaceTexture>()
            .expect("Surface texture should exist at this point");
        surface_texture.unwrap().present();

        self.world.resources.remove::<RenderSurfaceTextureView>();
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

        self.create_event(event);
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

        self.create_event(event);
    }
}

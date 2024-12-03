use crate::resources::Time;
use crate::system::{Commands, System, SystemHandler, SystemStage, SystemsContext};
use crate::window::{AppHandler, AppState, RenderContext, Renderer};
use crate::world::World;

use super::Events;

pub struct App {
    system_handler: SystemHandler,

    world: World,
    events: Events,
    // plugins: Plugins,
}

impl App {
    pub fn build() -> Self {
        let mut app = App {
            system_handler: SystemHandler::new(),
            world: World::new(),
            events: Events::new(),
        };

        return app;
    }

    pub fn create_event<T: 'static>(&mut self, event: T) {
        self.events.write(event);
    }

    /// Add a system to the startup stage
    pub fn add_startup_system(mut self, system: System) -> Self {
        self.system_handler.register_system(system, SystemStage::Startup);
        self
    }

    /// Add a system to the update stage
    pub fn add_system(mut self, system: System) -> Self {
        self.system_handler.register_system(system, SystemStage::Update);
        self
    }

    /// Register a system to a specific stage
    pub fn register_system(mut self, system: System, stage: SystemStage) -> Self {
        self.system_handler.register_system(system, stage);
        self
    }

    fn run_systems(&mut self, stage: SystemStage, renderer: Renderer) {
        let systems = self.system_handler.get_systems(stage);
        if systems.is_empty() {
            return;
        }

        let commands = Commands::build(&self.world);
        let mut ctx = SystemsContext::new(commands, &mut self.world.resources, &mut self.events, renderer);

        for system in systems.iter_mut() {
            system.run(&mut ctx, &mut self.world.entities);
        }

        ctx.commands.apply(&mut self.world);
    }

    pub(crate) fn update(&mut self, state: &mut AppState) {
        let mut context = RenderContext::new_update_context(state);

        self.world.resources.get_mut::<Time>().unwrap().update();

        self.run_systems(SystemStage::PreRender, context.as_renderer());
        self.run_systems(SystemStage::Render, context.as_renderer());
        self.run_systems(SystemStage::PostRender, context.as_renderer());
    } 

    pub fn run(self) {
        let (event_loop, mut app) = AppHandler::init(self);
        event_loop.run_app(&mut app).unwrap();

        // let systems = &self.systems;
        // let eq = systems[0].func_ptr == systems[1].func_ptr;
        // println!("eq: {} | {} == {}", eq, systems[0].name, systems[1].name);
    }

    pub(crate) fn render(&mut self, state: &mut AppState) -> Result<(), wgpu::SurfaceError> {
        let mut context = RenderContext::new_render_context(state)?;

        self.run_systems(SystemStage::PreRender, context.as_renderer());
        self.run_systems(SystemStage::Render, context.as_renderer());
        self.run_systems(SystemStage::PostRender, context.as_renderer());

        self.run_systems(SystemStage::Last, context.as_renderer());
        self.events.apply();

        Ok(())
    }
}

use crate::resources::Time;
use crate::system::{System, SystemsContext, Commands};
use crate::window::{AppHandler, AppState, RenderContext, Renderer};
use crate::world::World;

use super::Events;

pub struct App {
    startup_systems: Vec<System>,
    systems: Vec<System>,

    world: World,
    events: Events,
    // plugins: Plugins,
}

impl App {
    pub fn build() -> Self {
        let mut app = App {
            startup_systems: Vec::new(),
            systems: Vec::new(),
            world: World::new(),
            events: Events::new(),
        };

        return app;
    }

    pub fn create_event<T: 'static>(&mut self, event: T) {
        self.events.write(event);
    }

    pub fn add_startup_system(mut self, system: System) -> Self {
        self.startup_systems.push(system);
        self
    }

    pub fn add_system(mut self, system: System) -> Self {
        self.systems.push(system);
        self
    }

    pub(crate) fn run_startup_systems(&mut self, renderer: Renderer) {
        if self.startup_systems.is_empty() {
            return;
        }

        let commands = Commands::build(&self.world);
        let mut ctx = SystemsContext::new(commands, &mut self.world.resources, &mut self.events, renderer);

        for mut system in self.startup_systems.drain(..) {
            system.run(&mut ctx, &mut self.world.entities);
        }

        ctx.commands.apply(&mut self.world);
    }

    pub(crate) fn run_systems(&mut self, renderer: Renderer) {
        let commands = Commands::build(&self.world);
        let mut ctx = SystemsContext::new(commands, &mut self.world.resources, &mut self.events, renderer);

        for system in self.systems.iter_mut() {
            system.run(&mut ctx, &mut self.world.entities);
        }

        ctx.commands.apply(&mut self.world);
    }

    pub(crate) fn update(&mut self) {
        self.world.resources.get_mut::<Time>().unwrap().update();
    } 

    pub fn run(self) {
        let (event_loop, mut app) = AppHandler::init(self);
        event_loop.run_app(&mut app).unwrap();

        // let systems = &self.systems;
        // let eq = systems[0].func_ptr == systems[1].func_ptr;
        // println!("eq: {} | {} == {}", eq, systems[0].name, systems[1].name);
    }

    pub(crate) fn render(&mut self, state: &mut AppState) -> Result<(), wgpu::SurfaceError> {
        let mut context = RenderContext::new(state)?;

        self.run_startup_systems(context.as_renderer());
        self.run_systems(context.as_renderer());
        self.events.apply();

        Ok(())
    }
}

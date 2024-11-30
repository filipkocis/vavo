use crate::commands::Commands;
use crate::system::{System, SystemsContext};
use crate::world::World;

pub struct App {
    startup_systems: Vec<System>,
    systems: Vec<System>,

    world: World,
    // plugins: Plugins,
}

impl App {
    pub fn build() -> Self {
        let mut app = App {
            startup_systems: Vec::new(),
            systems: Vec::new(),
            world: World::new(),
        };

        return app;
    }

    pub fn add_startup_system(mut self, system: System) -> Self {
        self.startup_systems.push(system);
        self
    }

    pub fn add_system(mut self, system: System) -> Self {
        self.systems.push(system);
        self
    }

    fn run_startup_systems(&mut self) {
        let commands = Commands::build(&self.world);
        let mut ctx = SystemsContext::new(commands, &mut self.world.resources);

        for system in self.startup_systems.iter_mut() {
            system.run(&mut ctx, &mut self.world.entities);
        }

        ctx.commands.apply(&mut self.world);
    }

    fn run_systems(&mut self) {
        let commands = Commands::build(&self.world);
        let mut ctx = SystemsContext::new(commands, &mut self.world.resources);

        for system in self.systems.iter_mut() {
            system.run(&mut ctx, &mut self.world.entities);
        }

        ctx.commands.apply(&mut self.world);
    }

    pub fn run(&mut self) {
        self.run_startup_systems();

        loop {
            self.run_systems();
        }

        // let systems = &self.systems;
        // let eq = systems[0].func_ptr == systems[1].func_ptr;
        // println!("eq: {} | {} == {}", eq, systems[0].name, systems[1].name);
    }
}

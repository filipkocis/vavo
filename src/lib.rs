mod app;
mod commands;
mod entities;
mod query;
mod system;
mod world;
mod resources;

use app::App;
use query::{Query, RunQuery};
use system::System;

// impl World {
//     fn query<T: 'static>(&self) -> Vec<&mut T> {
//         let type_id = TypeId::of::<T>();
//         if let Some(archetypes) = self.archetypes.get_mut(&type_id) {
//             let components = archetypes.components.get_mut(&type_id).unwrap();
//             archetypes.ite().map(|c| c.downcast_ref::<T>().unwrap()).collect()
//         } else {
//             Vec::new()
//         }
//     }
// }

struct Position {
    x: f32,
    y: f32,
}
struct Velocity {
    dx: f32,
    dy: f32,
}

fn move_system<'a>(mut query: Query<'a, (&'a mut Position, &'a mut Velocity)>) {
    // let mut query = entities.query::<(&mut Position, &Velocity)>();
    // let mut query: Vec<(&mut Position, &mut Velocity)> = entities.iter_mut();
    for (position, velocity) in query.iter_mut() {
        position.x += velocity.dx;
        position.y += velocity.dy;
    }

    println!("move system finished");
}

fn log_system(_: Query<()>) {
    println!("Log system");
}

fn debug_system(_: Query<()>) {
    println!("Debug system");
}

fn run_system(_: Query<()>) {
    println!("Run system");
}

pub fn main() {
    App::build()
        .add_system(System::new("move", move_system))
        .add_system(System::new("move_2", move_system))
        .add_system(System::new("log", log_system))
        .add_system(System::new("debug", debug_system))
        .add_system(System::new("run", run_system))
        .run();

    let mut last_frame = std::time::Instant::now();
    let fps_target = 60.0;

    loop {
        let now = std::time::Instant::now();
        let delta = now.duration_since(last_frame).as_secs_f32();

        let fps = 1.0 / delta;
        if fps > fps_target {
            std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / fps_target - delta));
        }
        let now = std::time::Instant::now();
        let delta = now.duration_since(last_frame).as_secs_f32();
        let fps = 1.0 / delta;
        println!("FPS: {}, DELTA: {}", fps, delta);

        // engine.run();  // Run all systems
        last_frame = now;
    }
}

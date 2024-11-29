use engine::prelude::*;

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

fn main() {
    App::build()
        .add_system(System::new("move", move_system))
        .add_system(System::new("move_2", move_system))
        .add_system(System::new("log", log_system))
        .add_system(System::new("debug", debug_system))
        .add_system(System::new("run", run_system))
        .run();
}

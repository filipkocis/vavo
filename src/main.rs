use engine::prelude::*;

struct Position {
    x: f32,
    y: f32,
}
struct Velocity {
    dx: f32,
    dy: f32,
}
struct Foo;

fn move_system<'a>(_: &mut SystemsContext, mut query: Query<'a, (&'a mut Position, &'a mut Velocity)>) {
    for (position, velocity) in query.iter_mut() {
        position.x += velocity.dx;
        position.y += velocity.dy;

        println!("Pos: ({}, {}) Vel: ({}, {})", position.x, position.y, velocity.dx, velocity.dy);
    }
}

fn log_system(_: &mut SystemsContext, _: Query<&Foo>) {
    println!("Log system");
}

fn startup_system(ctx: &mut SystemsContext, _: Query<()>) {
    ctx.commands.spawn_empty()
        .insert(Position { x: 0.0, y: 0.0 })
        .insert(Velocity { dx: 1.0, dy: 1.0 });

    ctx.commands.spawn_empty()
        .insert(Position { x: 10.0, y: 10.0 })
        .insert(Velocity { dx: 0.1, dy: 0.1 })
        .insert(Foo);

    println!("startup system finished");
}

fn main() {
    App::build()
        .add_startup_system(System::new("startup", startup_system))
        .add_system(System::new("move", move_system))
        // .add_system(System::new("move_2", move_system))
        // .add_system(System::new("log", log_system))
        .run();
}

mod app;
mod query;
mod system;
mod world;
mod assets;
mod window;
mod renderer;
mod resources;

pub use renderer::shapes;
pub use renderer::palette;

pub mod prelude;

use app::App;

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

fn main() {
    App::build().run();

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

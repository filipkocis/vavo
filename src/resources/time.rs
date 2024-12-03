use std::time::Instant;

pub struct Time {
    start: Instant,
    last_frame: Instant,
    delta: f32,
}

impl Time {
    pub fn new() -> Self {
        let start = Instant::now();
        let last_frame = start;
        let delta = 0.0;

        Self { start, last_frame, delta }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn elapsed(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    pub fn fps(&self) -> f32 {
        1.0 / self.delta
    }

    pub fn sleep(&mut self, fps_target: f32) {
        let fps = self.fps();
        if fps > fps_target {
            std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / fps_target - self.delta));
        }
        self.update();
    }
}

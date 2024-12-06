use std::time::Instant;

pub struct Time {
    tick: u64,
    start: Instant,
    last_frame: Instant,
    delta: f32,
}

impl Time {
    pub fn new() -> Self {
        let start = Instant::now();
        let last_frame = start;
        let delta = 0.0;
        let tick = 0;

        Self { tick, start, last_frame, delta }
    }

    /// Update the delta time and last frame time, increment tick
    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.tick += 1;
    }

    pub fn start(&self) -> Instant {
        self.start
    }

    pub fn last_frame(&self) -> Instant {
        self.last_frame
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub(crate) fn tick_raw(&self) -> *const u64 {
        &self.tick
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

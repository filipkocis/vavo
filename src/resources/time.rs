use std::time::{Duration, Instant};

pub struct Time {
    /// Current world tick / frame count
    tick: u64,
    /// Application startup time
    start: Instant,
    /// The exact time the last fraem was rendered
    last_frame: Instant,
    /// Duration since the last frame
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

pub enum TimerVariant {
    Once,
    Repeat,
    RepeatN(u32),
}

pub struct Timer {
    elapsed: f32,
    duration: Duration,
    variant: TimerVariant,
    just_finished: bool,
}

impl Timer {
    pub fn new(duration: Duration, variant: TimerVariant) -> Self {
        Self { 
            elapsed: 0.0, 
            duration, 
            variant, 
            just_finished: false,
        } 
    }

    /// Total elapsed time since the timer started
    ///
    /// # Note
    /// Elapsed time is accumulated through `self.update(delta)` so it's not a completely accurate
    /// way to measure time
    pub fn elapsed(&self) -> Duration {
        Duration::from_secs_f32(self.elapsed)
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    pub fn finished(&self) -> bool {
        if self.just_finished {
            return true;
        }

        match self.variant {
            TimerVariant::Once => self.elapsed() >= self.duration,
            TimerVariant::Repeat => false,
            TimerVariant::RepeatN(n) => self.elapsed() >= self.duration * n,
        }
    }

    /// Returns true if the last update caused the timer to finish
    pub fn just_finished(&self) -> bool {
        self.just_finished
    }

    pub fn update(&mut self, delta: f32) {
        if self.finished() {
            self.just_finished = false;
            return;
        }

        self.elapsed += delta;

        if self.finished() {
            self.just_finished = true;
        }
    }
}

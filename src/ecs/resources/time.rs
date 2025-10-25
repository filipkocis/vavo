use crate::macros::Resource;
use web_time::{Duration, Instant};

#[derive(Resource)]
pub struct Time {
    /// Current world tick / frame count
    tick: u64,
    /// Application startup time
    start: Instant,
    /// The exact time the last frame was rendered
    last_frame: Instant,
    /// Duration since the last frame
    delta: f32,
}

impl Default for Time {
    fn default() -> Self {
        let start = Instant::now();
        let last_frame = start;

        Self {
            tick: 0,
            start,
            last_frame,
            delta: 0.0,
        }
    }
}

impl Time {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the delta time and last frame time, increment tick
    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.tick += 1;
    }

    /// Returns the start time of the application
    pub fn start(&self) -> Instant {
        self.start
    }

    /// Returns the start of the current frame (last frame render end)
    pub fn last_frame(&self) -> Instant {
        self.last_frame
    }

    /// Returns the current tick / frame count
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Returns the duration of the last frame in seconds
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Returns the elapsed time since the application started in seconds
    pub fn elapsed(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    /// Returns the frames per second (FPS) of the last frame
    pub fn fps(&self) -> f32 {
        1.0 / self.delta
    }

    /// Sleep the thread to achieve a target frame rate. If `fps <= fps_target` it will do nothing.
    /// This should realy only be used once, at the end of the frame, since it will block the
    /// thread and not call [`Self::update`], so each call to this function will sleep the same.
    /// It's not very accurate since it's based on the delta time of the last frame.
    pub fn sleep(&mut self, fps_target: f32) {
        let fps = self.fps();
        if fps > fps_target {
            let secs = 1.0 / fps_target - self.delta;
            std::thread::sleep(std::time::Duration::from_secs_f32(secs));
        }
    }
}

/// Resource used for fixed time step updates. It will try to run the systems on average at a fixed
/// rate, therefore it may run multiple times or zero times during udpate loop depending on the frame rate.
#[derive(Resource)]
pub struct FixedTime {
    time: Time,
    fixed_delta: f32,
    accumulator: f32,
}

impl FixedTime {
    pub fn new(fixed_delta: f32) -> Self {
        let time = Time::new();
        let accumulator = 0.0;

        Self {
            time,
            fixed_delta,
            accumulator,
        }
    }

    /// Create a new `FixedTime` with time step of `1.0 / hz`
    pub fn from_hz(hz: f32) -> Self {
        Self::new(1.0 / hz)
    }

    /// Increment the accumulator by the internal time's delta time
    pub(crate) fn update(&mut self) {
        self.time.update();
        self.accumulator += self.time.delta();
    }

    pub fn set_fixed_delta(&mut self, fixed_delta: f32) {
        self.fixed_delta = fixed_delta;
    }

    pub fn fixed_delta(&self) -> f32 {
        self.fixed_delta
    }

    /// Consume the accumulator and return the number of iterations necessary to reach the fixed
    /// time average
    pub fn iter(&mut self) -> usize {
        let mut iter = 0;

        while self.accumulator >= self.fixed_delta {
            iter += 1;
            self.accumulator -= self.fixed_delta;
        }

        iter
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

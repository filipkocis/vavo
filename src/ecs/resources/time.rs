use crate::macros::Resource;
use web_time::{Duration, Instant};

/// Resource used for tracking time in the application
#[derive(Resource, Debug, Clone)]
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
    /// Create a new Time resource
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the delta time and last frame time, increment tick
    #[inline]
    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.tick += 1;
    }

    /// Returns the start time of the application
    #[inline]
    pub fn start(&self) -> Instant {
        self.start
    }

    /// Returns the start of the current frame (last frame render end)
    #[inline]
    pub fn last_frame(&self) -> Instant {
        self.last_frame
    }

    /// Returns the current tick / frame count
    #[inline]
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Returns the duration of the last frame in seconds
    #[inline]
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Returns the elapsed time since the application started in seconds
    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    /// Returns the frames per second (FPS) of the last frame
    #[inline]
    pub fn fps(&self) -> f32 {
        1.0 / self.delta
    }

    /// Sleep the thread to achieve a target frame rate. If `fps <= fps_target` it will do nothing.
    /// This should realy only be used once, at the end of the frame, since it will block the
    /// thread and not call [`Self::update`], so each call to this function will sleep the same.
    /// It's not very accurate since it's based on the delta time of the last frame.
    #[inline]
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
#[derive(Resource, Debug, Clone)]
pub struct FixedTime {
    time: Time,
    fixed_delta: f32,
    accumulator: f32,
}

impl FixedTime {
    /// Create a new FixedTime with `fixed_delta` time step, (e.g. 60fps = 1.0 / 60.0)
    #[inline]
    pub fn new(fixed_delta: f32) -> Self {
        let time = Time::new();
        let accumulator = 0.0;

        Self {
            time,
            fixed_delta,
            accumulator,
        }
    }

    /// Create a new FixedTime from a target hertz (e.g 60fps = 60.0)
    #[inline]
    pub fn from_hz(hz: f32) -> Self {
        Self::new(1.0 / hz)
    }

    /// Update the internal time and accumulator
    /// # Note
    /// This should be called once per frame
    #[inline]
    pub(crate) fn update(&mut self) {
        self.time.update();
        self.accumulator += self.time.delta();
    }

    /// Sets the internal fixed delta time step
    #[inline]
    pub fn set_fixed_delta(&mut self, fixed_delta: f32) {
        self.fixed_delta = fixed_delta;
    }

    /// Returns the internal fixed delta time step
    #[inline]
    pub fn fixed_delta(&self) -> f32 {
        self.fixed_delta
    }

    /// Consume the accumulator and return the number of iterations necessary to reach the fixed
    /// time average
    #[inline]
    pub fn iter(&mut self) -> usize {
        let mut iter = 0;

        while self.accumulator >= self.fixed_delta {
            iter += 1;
            self.accumulator -= self.fixed_delta;
        }

        iter
    }
}

/// Resoruce used for tracking the FPS over time
#[derive(Default, Resource)]
pub struct FpsCounter {
    history: Vec<f32>,
    index: usize,
    sum: f32,
    capacity: usize,
    time: Time,
}

impl FpsCounter {
    /// Crate a new FpsCounter with a given history capacity
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            history: Vec::with_capacity(capacity),
            capacity,
            ..Default::default()
        }
    }

    /// Update the FPS counter with the latest FPS value
    #[inline]
    pub fn update(&mut self) {
        self.time.update();
        let fps = self.time.fps();

        if self.history.len() < self.capacity {
            self.history.push(fps);
            self.sum += fps;
        } else {
            self.sum -= self.history[self.index];
            self.history[self.index] = fps;
            self.sum += fps;
            self.index = (self.index + 1) % self.capacity;
        }
    }

    /// Returns the average FPS over the history
    #[inline]
    pub fn average_fps(&self) -> f32 {
        if self.history.is_empty() {
            0.0
        } else {
            self.sum / self.history.len() as f32
        }
    }

    /// Returns the last recorded FPS value
    #[inline]
    pub fn last_fps(&self) -> f32 {
        if self.history.is_empty() {
            0.0
        } else {
            self.history[(self.index + self.capacity - 1) % self.capacity]
        }
    }
}

/// Variant of the [Timer]
#[derive(Default, Clone, Copy, Debug)]
pub enum TimerVariant {
    /// Timer that runs once and then stops
    #[default]
    Once,
    /// Timer that repeats indefinitely
    Repeat,
    /// Timer that repeats a fixed number of times
    RepeatN(u32),
}

/// Timer utility for tracking elapsed time, with different variatns. The timer needs to be
/// manually updated each frame with the delta time or the timer won't progress.
#[derive(Resource, Clone, Debug)]
pub struct Timer {
    duration: Duration,
    variant: TimerVariant,
    elapsed: f32,
    repeats: u32,
    just_finished: bool,
    finished: bool,
}

impl Timer {
    /// Create a new Timer
    #[inline]
    pub fn new(duration: Duration, variant: TimerVariant) -> Self {
        Self {
            duration,
            variant,
            elapsed: 0.0,
            repeats: 0,
            just_finished: false,
            finished: false,
        }
    }

    /// Create a new one-shot Timer
    #[inline]
    pub fn once(duration: Duration) -> Self {
        Self::new(duration, TimerVariant::Once)
    }

    /// Create a new repeating Timer
    #[inline]
    pub fn repeating(duration: Duration) -> Self {
        Self::new(duration, TimerVariant::Repeat)
    }

    /// Create a new Timer that repeats N times
    #[inline]
    pub fn repeat_n(duration: Duration, n: u32) -> Self {
        Self::new(duration, TimerVariant::RepeatN(n))
    }

    /// Returns the variant of the timer
    #[inline]
    pub fn variant(&self) -> TimerVariant {
        self.variant
    }

    /// Total elapsed time since the timer started
    ///
    /// # Note
    /// Elapsed time is accumulated through `self.update(delta)` so it's not a completely accurate
    /// way to measure time.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        Duration::from_secs_f32(self.elapsed)
    }

    /// Resets the timer elapsed time to zero
    #[inline]
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.just_finished = false;
        self.finished = false;
        self.repeats = 0;
    }

    /// Returns the number of repeats completed for [TimerVariant::RepeatN]
    #[inline]
    pub fn repeats(&self) -> u32 {
        self.repeats
    }

    /// Returns the duration of the timer
    #[inline]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns true if the timer has finished based on its variant, or if it has just finished. In
    /// case of [TimerVariant::Repeat] this will never stay true.
    #[inline]
    pub fn finished(&self) -> bool {
        // If fully finished or just finished
        if self.finished || self.just_finished {
            return true;
        }

        false
    }

    /// Returns true if the last update caused the timer to finish. This will only be true for one
    /// update cycle after finishing.
    #[inline]
    pub fn just_finished(&self) -> bool {
        self.just_finished
    }

    /// Update the timer with the delta time in seconds. This will progress the timer.
    #[inline]
    pub fn update(&mut self, delta: f32) {
        self.just_finished = false;

        if self.finished {
            return;
        }

        self.elapsed += delta;

        if self.elapsed() >= self.duration {
            self.just_finished = true;

            match self.variant {
                TimerVariant::Once => {
                    // Mark as fully finished
                    self.finished = true;
                    // Clamp elapsed to duration
                    self.elapsed = self.duration.as_secs_f32();
                }
                TimerVariant::Repeat => {
                    // Wrap around elapsed time
                    self.elapsed = self.elapsed % self.duration.as_secs_f32();
                }
                TimerVariant::RepeatN(total) => {
                    self.repeats += 1;
                    if self.repeats >= total {
                        // Mark as fully finished
                        self.finished = true;
                        // Clamp elapsed to duration
                        self.elapsed = self.duration.as_secs_f32();
                    } else {
                        // Wrap around elapsed time
                        self.elapsed = self.elapsed % self.duration.as_secs_f32();
                    }
                }
            }
        }
    }
}

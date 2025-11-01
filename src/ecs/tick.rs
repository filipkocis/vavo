#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A tick is a simple counter used to track the number of updates or frames in a game loop. Used
/// heavily in the ECS for change detection.
pub struct Tick {
    value: u64,
}

impl Tick {
    #[inline]
    /// Create new tick with a starting value
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    #[inline]
    /// Get the current tick value
    pub fn get(&self) -> u64 {
        self.value
    }

    #[inline]
    /// Set the tick value
    pub fn set(&mut self, value: u64) {
        self.value = value;
    }

    #[inline]
    /// Increment the tick value by 1
    pub fn increment(&mut self) {
        self.value += 1;
    }
}

/// A struct that holds the timestamps for Changed and Added filters
pub struct TickStamp {
    changed: *const Tick,
    added: *const Tick,
    current: Tick,
    last_run: Tick,
}

impl TickStamp {
    #[inline]
    pub fn new(changed: &Tick, added: &Tick, current: Tick, last_run: Tick) -> Self {
        Self {
            changed,
            added,
            current,
            last_run,
        }
    }

    /// Returns the tick of the last change.
    #[inline]
    pub fn changed(&self) -> u64 {
        unsafe { *self.changed }.get()
    }

    /// Returns the tick when the value was added.
    #[inline]
    pub fn added(&self) -> u64 {
        unsafe { *self.added }.get()
    }

    /// Returns the current tick.
    #[inline]
    pub fn current_tick(&self) -> u64 {
        self.current.get()
    }

    /// Returns the tick of the last system run.
    #[inline]
    pub fn last_run(&self) -> u64 {
        self.last_run.get()
    }

    /// Sets the last run tick.
    #[inline]
    pub(crate) fn set_last_run(&mut self, tick: Tick) {
        self.last_run = tick;
    }
}

/// A struct that holds mutable timestamps for Changed and Added filters
/// For immutable, see [`Ticks`]
pub struct TickStampMut {
    changed: *mut Tick,
    added: *mut Tick,
    current: Tick,
    last_run: Tick,
}

impl TickStampMut {
    #[inline]
    pub fn new(changed: &mut Tick, added: &mut Tick, current: Tick, last_run: Tick) -> Self {
        Self {
            changed,
            added,
            current,
            last_run,
        }
    }

    /// Returns the tick of the last change.
    #[inline]
    pub fn changed(&self) -> u64 {
        unsafe { *self.changed }.get()
    }

    /// Returns the tick when the value was added.
    #[inline]
    pub fn added(&self) -> u64 {
        unsafe { *self.added }.get()
    }

    /// Returns the current tick.
    #[inline]
    pub fn current_tick(&self) -> u64 {
        self.current.get()
    }

    /// Returns the tick of the last system run.
    #[inline]
    pub fn last_run(&self) -> u64 {
        self.last_run.get()
    }

    /// Marks the changed tick as the current tick.
    #[inline]
    pub fn mark_changed(&mut self) {
        unsafe { *self.changed = self.current };
    }

    /// Sets the last run tick.
    #[inline]
    pub(crate) fn set_last_run(&mut self, tick: Tick) {
        self.last_run = tick;
    }
}

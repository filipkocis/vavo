#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A tick is a simple counter used to track the number of updates or frames in a game loop.
pub struct Tick {
    value: u64,
}

impl Tick {
    #[inline]
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    #[inline]
    pub fn get(&self) -> u64 {
        self.value
    }

    #[inline]
    pub fn set(&mut self, value: u64) {
        self.value = value;
    }

    #[inline]
    pub fn increment(&mut self) {
        self.value += 1;
    }
}

/// A struct that holds the timestamps for Changed and Added filters
pub struct TickStamp {
    changed: *const Tick,
    added: *const Tick,
    current: Tick,
}

impl TickStamp {
    #[inline]
    pub fn new(changed: &Tick, added: &Tick, current: Tick) -> Self {
        Self {
            changed,
            added,
            current,
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

    /// Checks if hte value was changed in the current tick.
    #[inline]
    pub fn is_changed(&self) -> bool {
        self.changed() == self.current.get()
    }

    /// Checks if the value was added in the current tick.
    #[inline]
    pub fn is_added(&self) -> bool {
        self.added() == self.current.get()
    }
}

/// A struct that holds mutable timestamps for Changed and Added filters
/// For immutable, see [`Ticks`]
pub struct TickStampMut {
    changed: *mut Tick,
    added: *mut Tick,
    current: Tick,
}

impl TickStampMut {
    #[inline]
    pub fn new(changed: &mut Tick, added: &mut Tick, current: Tick) -> Self {
        Self {
            changed,
            added,
            current,
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

    /// Marks the changed tick as the current tick.
    #[inline]
    pub fn mark_changed(&mut self) {
        unsafe { *self.changed = self.current };
    }

    /// Checks if hte value was changed in the current tick.
    #[inline]
    pub fn is_changed(&self) -> bool {
        self.changed() == self.current.get()
    }

    /// Checks if the value was added in the current tick.
    #[inline]
    pub fn is_added(&self) -> bool {
        self.added() == self.current.get()
    }
}

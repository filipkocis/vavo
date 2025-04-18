#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A tick is a simple counter used to track the number of updates or frames in a game loop.
pub struct Tick {
    value: usize,
}

impl Tick {
    #[inline]
    pub fn new(value: usize) -> Self {
        Self { value }
    }

    #[inline]
    pub fn get(&self) -> usize {
        self.value
    }

    #[inline]
    pub fn set(&mut self, value: usize) {
        self.value = value;
    }

    #[inline]
    pub fn increment(&mut self) {
        self.value += 1;
    }
}

/// A struct that holds the tick values for Changed and Added filters
pub struct Ticks {
    changed: *mut Tick,
    added: *const Tick,
    current: Tick,
}

impl Ticks {
    #[inline]
    pub fn new(changed: &mut Tick, added: &Tick, current: Tick) -> Self {
        Self {
            changed,
            added,
            current
        }
    }

    /// Returns the tick of the last change.
    #[inline]
    pub fn changed(&self) -> usize {
        unsafe { *self.changed }.get()
    }

    /// Returns the tick when the value was added.
    #[inline]
    pub fn added(&self) -> usize {
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

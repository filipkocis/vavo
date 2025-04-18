#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

    #[inline]
    pub fn as_ptr(&self) -> *const Tick {
        self as _
    }

    #[inline]
    pub fn as_mut(&mut self) -> *mut Tick {
        self as _
    }
}

pub struct Tick(pub(crate) u64);

impl Tick {
    pub(crate) fn new() -> Self {
        Self(0)
    }

    pub(crate) fn tick(&mut self) {
        self.0 += 1;
    }

    pub fn get(&self) -> u64 {
        self.0
    }

    pub(crate) fn reset(&mut self) {
        self.0 = 0;
    }
}

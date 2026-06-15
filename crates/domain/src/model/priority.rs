/// Priority value object: higher value = higher priority when resolving conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Priority(u32);

impl Priority {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self(0)
    }
}

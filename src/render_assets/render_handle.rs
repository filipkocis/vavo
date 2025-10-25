use std::{fmt::Debug, hash::Hash};

/// Handle to a render asset resource
pub struct RenderHandle<T> {
    id: u64,
    _marker: std::marker::PhantomData<T>,
}

impl<T> RenderHandle<T> {
    pub(super) fn new(id: u64) -> Self {
        Self {
            id,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Hash for RenderHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Debug for RenderHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RenderHandle({})", self.id)
    }
}

impl<T> Clone for RenderHandle<T> {
    fn clone(&self) -> Self {
        Self::new(self.id)
    }
}

impl<T> PartialEq for RenderHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for RenderHandle<T> {}

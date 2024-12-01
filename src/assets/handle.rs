use std::{fmt::Debug, hash::Hash};

/// Handle to an asset resource
pub struct Handle<T> {
    id: u64,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Handle<T> {
    pub(super) fn new(id: u64) -> Self {
        Self {
            id,
            _marker: std::marker::PhantomData
        }
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssetHandle({})", self.id)
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self::new(self.id)
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Handle<T> {}

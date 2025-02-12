use std::{fmt::Debug, hash::Hash};

use super::Asset;

/// Handle to an asset resource
#[derive(crate::macros::Component)]
pub struct Handle<A: Asset> {
    id: u64,
    _marker: std::marker::PhantomData<A>,
}

impl<A: Asset> Handle<A> {
    pub(super) fn new(id: u64) -> Self {
        Self {
            id,
            _marker: std::marker::PhantomData
        }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }
}

impl<A: Asset> Hash for Handle<A> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<A: Asset> Debug for Handle<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssetHandle({})", self.id)
    }
}

impl<A: Asset> Clone for Handle<A> {
    fn clone(&self) -> Self {
        Self::new(self.id)
    }
}

impl<A: Asset> PartialEq for Handle<A> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<A: Asset> Eq for Handle<A> {}

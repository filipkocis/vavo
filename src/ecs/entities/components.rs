use std::any::TypeId;

use crate::{
    ecs::{
        ptr::{DataPtr, DataPtrMut},
        store::blob::BlobVec,
        tick::{TickStamp, TickStampMut},
    },
    prelude::Tick,
};

/// A type which can be used as an entity component in the ECS.
pub trait Component: Send + Sync + 'static {
    fn get_type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Holds type-erased components of one type in a row, and their metadata.
pub(crate) struct ComponentsData {
    /// TypeId of the stored components
    type_id: TypeId,
    /// Components storage
    data: BlobVec,
    /// Ticks marking the last change of a component at `index`
    changed_at: Vec<Tick>,
    /// Ticks marking the creation of a component at `index`
    added_at: Vec<Tick>,
}

impl ComponentsData {
    /// Create new empty components row of type `C`
    pub(crate) fn new<C: Component>() -> Self {
        let type_id = C::get_type_id();
        let data = BlobVec::new_type::<C>(0);

        Self {
            type_id,
            data,
            changed_at: Vec::new(),
            added_at: Vec::new(),
        }
    }

    #[inline]
    /// Returns the type id of the components
    pub fn get_type_id(&self) -> TypeId {
        self.type_id
    }

    #[inline]
    /// Returns the number of stored components
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    /// Returns immutable [`TickStamp`] for component at `index`.
    pub fn get_ticks(&self, i: usize, current_tick: Tick) -> TickStamp {
        TickStamp::new(&self.changed_at[i], &self.added_at[i], current_tick)
    }

    #[inline]
    /// Returns mutable [`TickStampMut`] for component at `index`.
    pub fn get_ticks_mut(&mut self, i: usize, current_tick: Tick) -> TickStampMut {
        TickStampMut::new(&mut self.changed_at[i], &mut self.added_at[i], current_tick)
    }

    #[inline]
    /// Returns immutable data for component at `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn get<C: Component>(&self, index: usize, current_tick: Tick) -> DataPtr<C> {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: type is correct and index is callers responsibility
        let ptr = unsafe { self.data.get(index) };
        DataPtr::new(ptr, self.get_ticks(index, current_tick))
    }

    #[inline]
    /// Returns mutable data for component at `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn get_mut<C: Component>(&mut self, index: usize, current_tick: Tick) -> DataPtrMut<C> {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: type is correct and index is callers responsibility
        let ptr = unsafe { self.data.get_mut(index) };
        DataPtrMut::new(ptr, self.get_ticks_mut(index, current_tick))
    }

    // TODO: refactor, no monomorphic function. Best case is remove T from DataPtr and from BlobVec
    // public api, move it up to archetype/queryrun and res/resmut itself
    #[inline]
    /// Removes component at `index` and returns `(component, changed_at, added_at)` tuple.
    pub fn remove<C: Component>(&mut self, index: usize) -> (C, Tick, Tick) {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: type is correct and index is callers responsibility
        (
            unsafe { self.data.remove(index) },
            self.changed_at.swap_remove(index),
            self.added_at.swap_remove(index),
        )
    }

    #[inline]
    /// Insert new component with its `changed_at` and `added_at` ticks.
    pub fn insert<C: Component>(&mut self, component: C, changed_at: Tick, added_at: Tick) {
        // Safety: type and value is correct
        unsafe {
            self.data.push(component);
        }

        self.changed_at.push(changed_at);
        self.added_at.push(added_at);
    }
}

use std::{alloc::Layout, any::TypeId, collections::HashMap};

use crate::{
    ecs::{
        ptr::{DataPtr, DataPtrMut, UntypedPtr, UntypedPtrLt},
        store::blob::{BlobVec, DropFn},
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

// /// Type registry for components.
// pub(crate) struct ComponentsRegistry {
//     store: HashMap<TypeId, ComponentInfo>,
// }

#[derive(Debug)]
/// Holds metadata about a component type.
pub(crate) struct ComponentInfo {
    pub type_id: TypeId,
    pub layout: Layout,
    pub drop: Option<DropFn>,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
/// Raw pointer wrapper around [`ComponentInfo`]
pub(crate) struct ComponentInfoPtr(*const ComponentInfo);

impl ComponentInfoPtr {
    #[inline]
    /// Create new ptr
    pub fn new(ptr: *const ComponentInfo) -> Self {
        Self(ptr)
    }

    #[inline]
    /// Returns a reference to the pointer
    pub fn as_ref(&self) -> &ComponentInfo {
        unsafe { &*self.0 }
    }

    #[inline]
    /// Drops a component/resource
    pub fn drop(&self, ptr: UntypedPtr) {
        if let Some(drop_fn) = self.as_ref().drop {
            unsafe { drop_fn(ptr.inner()) }
        }
    }
}

#[derive(Debug)]
/// Holds type-erased components of one type in a row, and their metadata.
pub(crate) struct ComponentsData {
    type_id: TypeId,
    /// Components storage
    data: BlobVec,
    /// Ticks marking the last change of a component at `index`
    changed_at: Vec<Tick>,
    /// Ticks marking the creation of a component at `index`
    added_at: Vec<Tick>,
}

impl ComponentsData {
    /// Create new empty components row based on its [`ComponentInfo`].
    pub(crate) fn new(info: ComponentInfoPtr) -> Self {
        let info = info.as_ref();
        let data = BlobVec::new(info.layout, info.drop, 0);

        Self {
            type_id: info.type_id,
            data,
            changed_at: Vec::new(),
            added_at: Vec::new(),
        }
    }

    #[inline]
    /// Check if component at `index` has changed at `tick`.
    pub fn has_changed(&self, index: usize, current_tick: Tick) -> bool {
        debug_assert!(index < self.len(), "Index out of bounds");
        self.changed_at[index] == current_tick
    }

    #[inline]
    /// Check if component at `index` was added at `tick`.
    pub fn was_added(&self, index: usize, current_tick: Tick) -> bool {
        debug_assert!(index < self.len(), "Index out of bounds");
        self.added_at[index] == current_tick
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
    /// Returns [`UntypedPtrLt`] for component at `index`. Useful for when you need just the data,
    /// and it needs to be tied with a lifetime.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn get_untyped_lt<'a>(&'a self, index: usize) -> UntypedPtrLt<'a> {
        debug_assert!(index < self.len(), "Index out of bounds");
        // Safety: index is callers responsibility
        let untyped = unsafe { self.data.get(index) };
        UntypedPtrLt::new(untyped)
    }

    #[inline]
    /// Returns immutable data for component at `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn get(&self, index: usize, current_tick: Tick) -> DataPtr {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: index is callers responsibility
        let ptr = unsafe { self.data.get(index) };
        DataPtr::new(ptr, self.get_ticks(index, current_tick))
    }

    #[inline]
    /// Returns mutable data for component at `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn get_mut(&mut self, index: usize, current_tick: Tick) -> DataPtrMut {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: index is callers responsibility
        let ptr = unsafe { self.data.get_mut(index) };
        DataPtrMut::new(ptr, self.get_ticks_mut(index, current_tick))
    }

    #[inline]
    /// Removes component at `index` and returns `(component, changed_at, added_at)` tuple.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> (UntypedPtr, Tick, Tick) {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: index is callers responsibility
        (
            unsafe { self.data.remove(index) },
            self.changed_at.swap_remove(index),
            self.added_at.swap_remove(index),
        )
    }

    #[inline]
    /// Sets component at `index` to `component` and updates its ticks to `added_at` since 'set' is
    /// considered a new addition.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    ///
    /// # Safety
    /// You must ensure the component is of the same type as this row.
    pub fn set(&mut self, index: usize, component: UntypedPtr, added_at: Tick) {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: callers responsibility
        unsafe {
            self.data.set(component, index);
        }

        self.changed_at[index] = added_at;
        self.added_at[index] = added_at;
    }

    #[inline]
    /// Insert new component with its `changed_at` and `added_at` ticks.
    ///
    /// # Safety
    /// You must ensure the component is of the same type as this row.
    pub fn insert(&mut self, component: UntypedPtr, changed_at: Tick, added_at: Tick) {
        // Safety: callers responsibility
        unsafe {
            self.data.push(component);
        }

        self.changed_at.push(changed_at);
        self.added_at.push(added_at);
    }
}

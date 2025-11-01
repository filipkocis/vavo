use std::{
    alloc::Layout,
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    ecs::{
        ptr::{DataPtr, DataPtrMut, OwnedPtr, UntypedPtrLt},
        store::blob::{BlobVec, DropFn, new_option_drop_fn},
        tick::{TickStamp, TickStampMut},
    },
    prelude::Tick,
};

/// A type which can be used as an entity component in the ECS.
pub trait Component: Send + Sync + 'static {
    /// Returns the `TypeId` of the component.
    #[inline]
    fn get_type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

#[repr(transparent)]
/// Mutable component reference.
/// Holds a raw mutable pointer to a component.
pub struct Mut<'a, C: Component>(pub(crate) DataPtrMut, PhantomData<&'a C>);

impl<'a, C: Component> Mut<'a, C> {
    /// Creates a new mutable component reference from a raw pointer.
    #[inline]
    pub(crate) fn new(data: DataPtrMut) -> Self {
        Self(data, PhantomData)
    }
}

impl<'a, C: Component> Deref for Mut<'a, C> {
    type Target = C;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.raw().cast::<C>() }
    }
}

impl<'a, C: Component> DerefMut for Mut<'a, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.mark_changed();
        // We just marked it as changed
        self.deref_mut_no_change()
    }
}

#[repr(transparent)]
/// Mutable component reference.
/// Holds a raw mutable pointer to a component.
pub struct Ref<'a, C: Component>(pub(crate) DataPtr, PhantomData<&'a C>);

impl<'a, C: Component> Ref<'a, C> {
    /// Creates a new mutable component reference from a raw pointer.
    #[inline]
    pub(crate) fn new(data: DataPtr) -> Self {
        Self(data, PhantomData)
    }
}

impl<'a, C: Component> Deref for Ref<'a, C> {
    type Target = C;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.raw().cast::<C>() }
    }
}

#[derive(Debug, Default)]
/// Type registry for components.
pub struct ComponentsRegistry {
    pub(crate) store: HashMap<TypeId, ComponentInfoPtr>,
}

impl ComponentsRegistry {
    /// Creates a new empty registry.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the [`ComponentInfo`] for a given type wrapped in a ptr.
    #[inline]
    pub fn get(&self, type_id: &TypeId) -> Option<ComponentInfoPtr> {
        self.store.get(type_id).copied()
    }

    /// Register a new component type.
    #[inline]
    fn register<C: Component>(&mut self) {
        let type_id = C::get_type_id();
        let layout = Layout::new::<C>();
        let drop = new_option_drop_fn::<C>();

        let info = ComponentInfo {
            type_id,
            layout,
            drop,
        };

        self.store.insert(info.type_id, ComponentInfoPtr::new(info));
    }

    /// Returns the [`ComponentInfo`] for a given type, if it doesn't exist it will register it.
    pub(crate) fn get_or_register<C: Component>(&mut self) -> ComponentInfoPtr {
        let type_id = C::get_type_id();

        if let Some(info) = self.get(&type_id) {
            info
        } else {
            self.register::<C>();
            // Safety: just registered
            self.store[&type_id]
        }
    }
}

impl Drop for ComponentsRegistry {
    fn drop(&mut self) {
        for (_, info) in self.store.drain() {
            unsafe {
                // Safety: we own the pointer and it was allocated with Box
                let _ = Box::from_raw(info.0 as *mut ComponentInfo);
            }
        }
    }
}

#[derive(Debug)]
/// Holds metadata about a component type.
pub struct ComponentInfo {
    pub type_id: TypeId,
    pub layout: Layout,
    pub drop: Option<DropFn>,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
/// Raw pointer wrapper around [`ComponentInfo`]
pub struct ComponentInfoPtr(*const ComponentInfo);

impl ComponentInfoPtr {
    /// Create new ptr from owned info
    #[inline]
    pub(crate) fn new(info: ComponentInfo) -> Self {
        let boxed = Box::new(info);
        Self(Box::into_raw(boxed))
    }

    /// Check if the pointer is null
    #[inline]
    pub(super) fn is_null(&self) -> bool {
        self.0.is_null()
    }

    /// Create a null pointer
    #[inline]
    pub(super) fn null() -> Self {
        Self(std::ptr::null())
    }

    /// Drops a component/resource
    #[inline]
    pub fn drop(&self, ptr: OwnedPtr) {
        if let Some(drop_fn) = self.as_ref().drop {
            unsafe { drop_fn(ptr.inner()) }
        }
    }
}

impl AsRef<ComponentInfo> for ComponentInfoPtr {
    /// Returns a reference to the underlying `ComponentInfo`.
    #[inline]
    fn as_ref(&self) -> &ComponentInfo {
        unsafe { &*self.0 }
    }
}

/// Holds owned component data, either removed from [ComponentsData] or newly created.
#[derive(Debug)]
pub struct UntypedComponentData<'a> {
    pub data: OwnedPtr<'a>,
    pub changed_at: Tick,
    pub added_at: Tick,
}

impl<'a> UntypedComponentData<'a> {
    pub fn new(data: OwnedPtr<'a>, changed_at: Tick, added_at: Tick) -> Self {
        Self {
            data,
            changed_at,
            added_at,
        }
    }
}

#[derive(Debug)]
/// Holds type-erased components of one type in a row, and their metadata.
pub struct ComponentsData {
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

    /// Check if component at `index` has changed since `tick`.
    #[inline]
    pub fn changed_since(&self, index: usize, tick: Tick) -> bool {
        debug_assert!(index < self.len(), "Index out of bounds");
        self.changed_at[index] > tick
    }

    /// Check if component at `index` was added since `tick`.
    #[inline]
    pub fn added_since(&self, index: usize, tick: Tick) -> bool {
        debug_assert!(index < self.len(), "Index out of bounds");
        self.added_at[index] > tick
    }

    /// Returns the type id of the components
    #[inline]
    pub fn get_type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the number of stored components
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if there are no stored components
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns immutable [`TickStamp`] for component at `index`.
    #[inline]
    pub fn get_ticks(&self, i: usize, current_tick: Tick, last_run: Tick) -> TickStamp {
        TickStamp::new(
            &self.changed_at[i],
            &self.added_at[i],
            current_tick,
            last_run,
        )
    }

    /// Returns mutable [`TickStampMut`] for component at `index`.
    #[inline]
    pub fn get_ticks_mut(&mut self, i: usize, current_tick: Tick, last_run: Tick) -> TickStampMut {
        TickStampMut::new(
            &mut self.changed_at[i],
            &mut self.added_at[i],
            current_tick,
            last_run,
        )
    }

    /// Returns [`UntypedPtrLt`] for component at `index`. Useful for when you need just the data,
    /// and it needs to be tied with a lifetime.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn get_untyped_lt<'a>(&'a self, index: usize) -> UntypedPtrLt<'a> {
        debug_assert!(index < self.len(), "Index out of bounds");
        // Safety: index is callers responsibility
        let untyped = unsafe { self.data.get(index) };
        UntypedPtrLt::new(untyped)
    }

    /// Set the `changed_at` tick for component at `index`.
    #[inline]
    pub fn set_changed_at(&mut self, index: usize, tick: Tick) {
        debug_assert!(index < self.len(), "Index out of bounds");
        self.changed_at[index] = tick;
    }

    /// Returns immutable data for component at `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn get(&self, index: usize, current_tick: Tick, last_run: Tick) -> DataPtr {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: index is callers responsibility
        let ptr = unsafe { self.data.get(index) };
        DataPtr::new(ptr, self.get_ticks(index, current_tick, last_run))
    }

    /// Returns mutable data for component at `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize, current_tick: Tick, last_run: Tick) -> DataPtrMut {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: index is callers responsibility
        let ptr = unsafe { self.data.get_mut(index) };
        DataPtrMut::new(ptr, self.get_ticks_mut(index, current_tick, last_run))
    }

    /// Swap-Removes component at `index` and returns the component data.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> UntypedComponentData<'_> {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: index is callers responsibility
        let component = unsafe { self.data.remove(index) };
        UntypedComponentData::new(
            component,
            self.changed_at.swap_remove(index),
            self.added_at.swap_remove(index),
        )
    }

    /// Sets component at `index` to `component` and updates its ticks to `added_at` since 'set' is
    /// considered a new addition.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    ///
    /// # Safety
    /// You must ensure the component is of the same type as this row.
    #[inline]
    pub fn set(&mut self, index: usize, component: UntypedComponentData) {
        debug_assert!(index < self.len(), "Index out of bounds");

        // Safety: callers responsibility
        unsafe {
            self.data.set(component.data, index);
        }

        self.changed_at[index] = component.added_at;
        self.added_at[index] = component.added_at;
    }

    /// Insert new component data at the end of the row.
    ///
    /// # Safety
    /// You must ensure the component is of the same type as this row.
    #[inline]
    pub fn insert(&mut self, component: UntypedComponentData) {
        // Safety: callers responsibility
        unsafe {
            self.data.push(component.data);
        }

        self.changed_at.push(component.changed_at);
        self.added_at.push(component.added_at);
    }
}

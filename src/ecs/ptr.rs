use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::ecs::tick::{TickStamp, TickStampMut};

#[repr(transparent)]
#[derive(Debug)]
/// Pointer to a component or resource, wrapper around NonNull<u8>
pub struct UntypedPtr {
    ptr: NonNull<u8>,
}

impl UntypedPtr {
    #[inline]
    /// Creates new untyped pointer
    pub fn new(ptr: NonNull<u8>) -> Self {
        Self { ptr }
    }

    #[inline]
    /// Creates new untyped pointer
    ///
    /// # Safety
    /// Pointer must be non-null and valid
    pub fn new_raw(ptr: *mut u8) -> Self {
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Self { ptr }
    }

    #[inline]
    /// Unwraps the pointer
    pub fn inner(self) -> NonNull<u8> {
        self.ptr
    }

    #[inline]
    /// Returns the internal pointer
    pub fn as_ptr(&self) -> &NonNull<u8> {
        &self.ptr
    }

    #[inline]
    /// Returns the internal pointer
    pub fn as_mut(&mut self) -> &mut NonNull<u8> {
        &mut self.ptr
    }
}

/// Same as [`UntypedPtr`], but with a lifetime
pub struct UntypedPtrLt<'a> {
    ptr: NonNull<u8>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> UntypedPtrLt<'a> {
    /// Wraps the pointer with a lifetime
    pub fn new(untyped: UntypedPtr) -> Self {
        Self {
            ptr: untyped.ptr,
            _marker: std::marker::PhantomData,
        }
    }

    #[inline]
    /// Returns the internal pointer
    pub fn as_ptr(&self) -> &NonNull<u8> {
        &self.ptr
    }

    #[inline]
    /// Returns the internal pointer
    pub fn as_mut(&mut self) -> &mut NonNull<u8> {
        &mut self.ptr
    }
}

/// Immutable data pointer to either a component or resource with its tick timestamps
pub struct DataPtr {
    ptr: UntypedPtr,
    stamp: TickStamp,
}

/// Mutable data pointer to either a component or resource with its tick timestamps.
/// Provides automatic change detection update on mutable deref
pub struct DataPtrMut {
    ptr: UntypedPtr,
    stamp: TickStampMut,
}

impl DataPtr {
    #[inline]
    /// Creates a new typed pointer from a (blob's) raw pointer and it's timestamps
    pub fn new(data: UntypedPtr, stamp: TickStamp) -> Self {
        Self { ptr: data, stamp }
    }

    #[inline]
    /// Returns the inner raw pointer
    pub(crate) fn raw(&self) -> *const u8 {
        self.ptr.as_ptr().as_ptr()
    }

    #[inline]
    /// Returns the timestamp of the last change to the data
    pub fn changed_at(&self) -> u64 {
        self.stamp.changed()
    }

    #[inline]
    /// Returns the timestamp of when the data was created
    pub fn added_at(&self) -> u64 {
        self.stamp.added()
    }
}

impl DataPtrMut {
    #[inline]
    /// Creates a new mutable typed pointer from a (blob's) raw pointer and it's timestamps
    pub fn new(data: UntypedPtr, stamp: TickStampMut) -> Self {
        Self { ptr: data, stamp }
    }

    #[inline]
    /// Returns the inner raw pointer
    pub(crate) fn raw(&self) -> *const u8 {
        self.ptr.as_ptr().as_ptr()
    }

    #[inline]
    pub fn changed_at(&self) -> u64 {
        self.stamp.changed()
    }

    #[inline]
    pub fn added_at(&self) -> u64 {
        self.stamp.added()
    }
}

impl<T> Deref for DataPtr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> Deref for DataPtrMut<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for DataPtrMut<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stamp.mark_changed();
        unsafe { &mut *self.ptr }
    }
}

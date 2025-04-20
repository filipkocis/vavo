use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::ecs::tick::{TickStamp, TickStampMut};

/// Immutable data pointer to either a component or resource with its tick timestamps
pub struct DataPtr<T> {
    ptr: *const T,
    stamp: TickStamp,
}

/// Mutable data pointer to either a component or resource with its tick timestamps.
/// Provides automatic change detection update on mutable deref
pub struct DataPtrMut<T> {
    ptr: *mut T,
    stamp: TickStampMut,
}

impl<T> DataPtr<T> {
    #[inline]
    /// Creates a new typed pointer from a (blob's) raw pointer and it's timestamps
    pub fn new(data: NonNull<u8>, stamp: TickStamp) -> Self {
        Self {
            ptr: data.cast::<T>().as_ptr(),
            stamp,
        }
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

impl<T> DataPtrMut<T> {
    #[inline]
    /// Creates a new mutable typed pointer from a (blob's) raw pointer and it's timestamps
    pub fn new(data: NonNull<u8>, stamp: TickStampMut) -> Self {
        Self {
            ptr: data.cast::<T>().as_ptr(),
            stamp,
        }
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

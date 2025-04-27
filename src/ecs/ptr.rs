use std::{marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use crate::ecs::tick::{TickStamp, TickStampMut};

#[repr(transparent)]
#[derive(Debug)]
/// Pointer to a component or resource
///
/// The inner pointer is:
/// - always non-null
/// - exclusively owned data but behind a reference like `&mut T`
/// - valid for the lifetime `'a`
/// - data must be dropped manually
pub struct OwnedPtr<'a> {
    ptr: NonNull<u8>,
    marker: PhantomData<&'a ()>,
}

impl<'a> OwnedPtr<'a> {
    /// Creates new owned pointer, you must ensure `ptr` is valid for `Self` requirements
    #[inline]
    pub unsafe fn from_raw(ptr: NonNull<u8>) -> Self {
        Self {
            ptr,
            marker: PhantomData,
        }
    }

    /// Creates new owned pointer, you must ensure `ptr` is valid for `Self` requirements
    /// The reference must not be used after this call
    #[inline]
    pub unsafe fn new_ref<T>(ptr: &'a mut ManuallyDrop<T>) -> OwnedPtr<'a> {
        let raw = &**ptr as *const T as _;
        let ptr = NonNull::new_unchecked(raw); // Safety: pointer is valid
        Self::from_raw(ptr) // Safety:
    }

    /// Consumes self and returns the inner pointer
    #[inline]
    pub fn inner(self) -> NonNull<u8> {
        self.ptr
    }

    /// Consumes self and reads the inner value as `T`
    ///
    /// # Safety
    /// The pointer must be valid for `T`
    #[inline]
    pub unsafe fn read<T>(self) -> T {
        self.inner().cast::<T>().read()
    }
}

#[repr(transparent)]
#[derive(Debug)]
/// Pointer to a component or resource, wrapper around NonNull<u8>
pub struct UntypedPtr {
    ptr: NonNull<u8>,
}

impl UntypedPtr {
    /// Creates new untyped pointer
    #[inline]
    pub fn from_raw(ptr: NonNull<u8>) -> Self {
        Self { ptr }
    }

    /// Unwraps the pointer
    #[inline]
    pub fn inner(self) -> NonNull<u8> {
        self.ptr
    }

    /// Returns the internal pointer
    #[inline]
    pub fn as_ptr(&self) -> &NonNull<u8> {
        &self.ptr
    }

    /// Returns the internal pointer
    #[inline]
    pub fn as_mut(&mut self) -> &mut NonNull<u8> {
        &mut self.ptr
    }
}

#[repr(transparent)]
/// Same as [`UntypedPtr`], but with a lifetime
pub struct UntypedPtrLt<'a> {
    ptr: NonNull<u8>,
    marker: PhantomData<&'a ()>,
}

impl<'a> UntypedPtrLt<'a> {
    /// Wraps the pointer with a lifetime
    pub fn new(untyped: UntypedPtr) -> Self {
        Self {
            ptr: untyped.ptr,
            marker: PhantomData,
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
    /// Marks this component as changed
    pub(crate) fn mark_changed(&mut self) {
        self.stamp.mark_changed();
    }

    #[inline]
    pub fn changed_at(&self) -> u64 {
        self.stamp.changed()
    }

    #[inline]
    pub fn added_at(&self) -> u64 {
        self.stamp.added()
    }

    /// Returns the current stamp tick.
    #[inline]
    pub fn current_stamp_tick(&self) -> u64 {
        self.stamp.current_tick()
    }
}

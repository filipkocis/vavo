use std::{
    alloc::{self, Layout},
    num::NonZero,
    ptr::NonNull,
};

pub type DropFn = unsafe fn(NonNull<u8>);

/// A blob vector is a contiguous block of memory that stores type-erased elements of one type.
pub struct BlobVec {
    layout: Layout,
    data: NonNull<u8>,
    len: usize,
    capacity: usize,
    drop: Option<DropFn>,
}

/// *Stolen* from unstable rust `Layout::repeat`
fn layout_repeat(layout: &Layout, n: usize) -> Option<Layout> {
    let padded = layout.pad_to_align();
    if let Some(size) = padded.size().checked_mul(n) {
        if size > isize::MAX as usize {
            return None;
        }
        let layout = unsafe { Layout::from_size_align_unchecked(size, padded.align()) };
        Some(layout)
    } else {
        None
    }
}

impl BlobVec {
    /// Create a new blob storage with the given type layout and capacity.
    pub fn new(layout: Layout, drop: Option<DropFn>, capacity: usize) -> Self {
        let data = Self::dangling(layout);

        let default_capacity = if layout.size() == 0 { usize::MAX } else { 0 };

        let mut blob = Self {
            layout,
            data,
            len: 0,
            capacity: default_capacity,
            drop,
        };

        blob.reserve(capacity);
        blob
    }

    /// Create a dangling pointer with proper alignment
    fn dangling(layout: Layout) -> NonNull<u8> {
        let align = NonZero::<usize>::new(layout.align()).expect("alignment must be > 0");
        debug_assert!(align.is_power_of_two(), "alignment must be power of two");
        NonNull::<u8>::dangling().with_addr(align)
    }

    /// Amount of elements stored in the blob
    pub fn len(&self) -> usize {
        self.len
    }

    /// Capacity of the blob
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Layout of the type which the blob's elements have
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Ensure that the blob has enough space for `additional` elements.
    ///
    /// # Panic
    /// Panics if the new capacity overflows `isize::MAX`
    pub fn reserve(&mut self, additional: usize) {
        let grow_by = self
            .len
            .checked_add(additional)
            .expect("Overflow in length")
            .saturating_sub(self.capacity);
        if grow_by > 0 {
            // Safety: `grow_by` is 0 for ZSTs, panics on overflow
            self.grow_by(grow_by);
        }
    }

    /// Reallocate the blob to a new capacity.
    /// Caller must ensure this is not called for ZSTs and that the new capacity is greater than 0
    unsafe fn reallocate(&mut self, new_capacity: usize) {
        debug_assert!(self.layout.size() != 0, "Cannot reallocate ZSTs");
        debug_assert!(new_capacity > 0, "New capacity must be greater than 0");

        if new_capacity as isize > isize::MAX {
            panic!("Capacity overflow");
        }

        let new_layout =
            layout_repeat(&self.layout, new_capacity).expect("Failed to repeat layout");
        let new_data = if self.capacity == 0 {
            // Safety: this function isn't called for ZSTs
            alloc::alloc(new_layout)
        } else {
            let old_layout =
                layout_repeat(&self.layout, self.capacity).expect("Failed to repeat layout");

            // Safety: ptr was allocated with the same allocator and layout_repeat
            // caller ensures new_size is not 0
            alloc::realloc(self.data.as_ptr(), old_layout, new_layout.size())
        };

        self.data = NonNull::new(new_data).unwrap_or_else(|| alloc::handle_alloc_error(new_layout));
        self.capacity = new_capacity;
    }

    /// Grow the blob by `additional` elements.
    ///
    /// # Panics
    /// Panics if the new capacity overflows usize or in case of ZSTs
    fn grow_by(&mut self, additional: usize) {
        let new_capacity = self
            .capacity
            .checked_add(additional)
            .expect("Overflow in capacity");

        if new_capacity > self.capacity {
            // Safety: cap is guaranteed to be > 0, and it's not called for ZSTs because
            // new_capacity would overflow
            unsafe {
                self.reallocate(new_capacity);
            }
        }
    }

    /// Wrapper for [`core::ptr::copy_nonoverlapping`]
    /// Caller must ensure valid non-overlapping pointers
    unsafe fn copy_nonoverlapping(&self, src: NonNull<u8>, dst: NonNull<u8>) {
        core::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), self.layout.size());
    }

    /// Wrapper for [`core::ptr::swap_nonoverlapping`]
    /// Caller must ensure valid non-overlapping pointers
    unsafe fn swap_nonoverlapping(&self, x: NonNull<u8>, y: NonNull<u8>) {
        core::ptr::swap_nonoverlapping(x.as_ptr(), y.as_ptr(), self.layout.size());
    }

    /// Get a pointer to the element at index `i`.
    /// Caller must ensure a valid index
    unsafe fn get_raw(&self, i: usize) -> NonNull<u8> {
        debug_assert!(i <= self.len, "Index out of bounds");
        self.get_raw_unchecked(i)
    }

    /// Get a pointer to the element at index `i`.
    /// Caller must ensure the index is within bounds.
    unsafe fn get_raw_unchecked(&self, i: usize) -> NonNull<u8> {
        let offset = i * self.layout.size();
        let ptr = self.data.as_ptr().add(offset);
        // Safety: pointer is non-null since data is non-null
        NonNull::new_unchecked(ptr)
    }

    /// Push a new element to the blob.
    /// New capacity can be greater than `isize::MAX`
    /// Value must be valid and can't be pointing to the blob
    unsafe fn push_raw(&mut self, value: NonNull<u8>) {
        self.reserve(1);
        // Safety: self.len is now within bounds
        let dst = self.get_raw_unchecked(self.len);
        self.len += 1;
        // Safety: dst and value are non-overlapping and valid
        self.copy_nonoverlapping(value, dst);
    }

    /// Swap remove the element at index `i`.
    /// Caller must ensure the index is within bounds.
    unsafe fn swap_remove_raw(&mut self, i: usize) -> NonNull<u8> {
        debug_assert!(i <= self.len, "Index out of bounds");
        let last = self.len - 1;
        let last_ptr = self.get_raw(last); // Safety: valid index

        if i != last {
            let i_ptr = self.get_raw(i); // Safety: caller
                                         // Safety: i != last, so they are non-overlapping
            self.swap_nonoverlapping(last_ptr, i_ptr);
        }

        self.len -= 1;
        last_ptr
    }

    /// Get a mutable slice of the blob.
    /// Caller must ensure the range is valid and within bounds.
    unsafe fn get_slice_raw(&self, start: usize, end: usize) -> &mut [u8] {
        debug_assert!(start < end, "Start index must be less than end index");
        debug_assert!(start <= self.len, "Start index out of bounds");
        debug_assert!(end <= self.len, "End index out of bounds");

        let start_ptr = self.get_raw(start);
        core::slice::from_raw_parts_mut(start_ptr.as_ptr(), end - start)
    }

    /// Shrink the blob to fit the given capacity.
    /// If the capacity is less than the current length, the excess elements are cleared.
    /// If the capacity is 0, the blob is deallocated and set to a dangling, otherwise he blob is
    /// reallocated to the new capacity.
    fn shrink_to_fit_raw(&mut self, cap: usize) {
        if self.layout.size() == 0 || cap >= self.capacity {
            return;
        }

        if cap < self.len {
            // Safety: the range is valid because it uses the current length
            unsafe {
                self.clear_range(cap, self.len);
            }
        }

        if cap == 0 {
            // Safety: the blob is empty
            unsafe {
                self.deallocate();
            }
        } else {
            // Safety: cap is > 0 and it's not a ZST (layout.size > 0)
            unsafe {
                self.reallocate(cap);
            }
        }
    }

    /// Shrink the blob to fit the given capacity.
    /// New capacity will not be lower than the current length.
    pub fn shrink_to(&mut self, cap: usize) {
        let cap = self.len.max(cap);

        if cap < self.capacity {
            self.shrink_to_fit_raw(cap);
        }
    }

    /// Shrink the blob to fit the current length.
    /// New capacity will be equal to the current length.
    pub fn shrink_to_fit(&mut self) {
        self.shrink_to(0);
    }

    /// Convert a value to a pointer.
    fn type_to_ptr<T>(value: T) -> NonNull<u8> {
        let ptr = Box::into_raw(Box::new(value)) as *mut u8;
        // Safety: The pointer is valid because it was created from a Box
        unsafe { NonNull::new_unchecked(ptr) }
    }

    /// Convert a pointer to a value.
    /// Caller must ensure the pointer is valid and aligned, and correct T
    unsafe fn ptr_to_type<T>(ptr: NonNull<u8>) -> T {
        ptr.cast::<T>().as_ptr().read()
    }

    /// Push a new element to the blob.
    ///
    /// # Safety
    /// Caller must ensure a correct type, value can't be pointing to the blob
    ///
    /// # Panic
    /// Panics if the new capacity overflows `isize::MAX`
    pub unsafe fn push<T>(&mut self, value: T) {
        let ptr = Self::type_to_ptr(value);
        self.push_raw(ptr); // Safety: caller
    }

    /// Remove an element from the blob.
    ///
    /// # Safety
    /// Caller must ensure a correct type and index
    pub unsafe fn remove<T>(&mut self, i: usize) -> T {
        let ptr = self.swap_remove_raw(i); // Safety: caller
        Self::ptr_to_type(ptr) // Safety: ptr is vald
    }

    /// Get a reference to an element
    ///
    /// # Safety
    /// Caller must ensure a correct type and index
    pub unsafe fn get<T>(&self, i: usize) -> &T {
        let ptr = self.get_raw(i);
        ptr.cast::<T>().as_ref()
    }

    /// Get a mutable reference to an element
    ///
    /// # Safety
    /// Caller must ensure a correct type and index
    pub unsafe fn get_mut<T>(&mut self, i: usize) -> &mut T {
        let ptr = self.get_raw(i);
        ptr.cast::<T>().as_mut()
    }

    /// Get a slice of the blob
    ///
    /// # Safety
    /// Caller must ensure a correct type and index
    pub unsafe fn get_slice<T>(&self, start: usize, end: usize) -> &[T] {
        let slice = self.get_slice_raw(start, end);
        core::mem::transmute(slice)
    }

    /// Get a mutable slice of the blob
    ///
    /// # Safety
    /// Caller must ensure a correct type and index
    pub unsafe fn get_slice_mut<T>(&mut self, start: usize, end: usize) -> &mut [T] {
        let slice = self.get_slice_raw(start, end);
        core::mem::transmute(slice)
    }

    /// Clear the blob
    pub fn clear(&mut self) {
        if self.len == 0 {
            return;
        }

        // Safety: the range is valid
        unsafe { self.clear_range(0, self.len) };
    }

    /// Drop elements from a range `start..end` in the blob.
    /// Caller must ensure the range is valid and within bounds.
    unsafe fn clear_range(&mut self, start: usize, end: usize) {
        debug_assert!(start < end, "Start index must be less than end index");

        if let Some(drop_fn) = self.drop {
            for i in start..end {
                // Safety: caller ensures the index is valid
                let ptr = self.get_raw_unchecked(i);
                drop_fn(ptr);
            }
        }

        self.len = start;
    }

    /// Deallocate the blob, does not call drop on the elements
    /// Caller must drop the elements (if any) manually
    unsafe fn deallocate(&mut self) {
        let layout = layout_repeat(&self.layout, self.capacity).expect("Failed to repeat layout");

        if layout.size() != 0 {
            // Safety: it was allocated with the same allocator and layout_repeat
            alloc::dealloc(self.data.as_ptr(), layout);
            self.data = Self::dangling(self.layout);
            self.capacity = 0;
        }
    }
}

impl Drop for BlobVec {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            // Safety: we cleared the blob
            self.deallocate();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::Layout;

    #[test]
    fn test_blob() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None, 2);
        assert_eq!(blob.len(), 0);
        assert_eq!(blob.capacity(), 2);
        assert_eq!(blob.layout().size(), 4);
        assert_eq!(blob.layout().align(), 4);
        assert_eq!(blob.layout().size(), std::mem::size_of::<u32>());

        unsafe {
            blob.push(1u32);
            blob.push(2u32);

            assert_eq!(blob.len(), 2);
            assert_eq!(blob.get::<u32>(0), &1);
            assert_eq!(blob.get::<u32>(1), &2);
            assert_eq!(blob.get_slice::<u32>(0, 2), &[1, 2]);
            assert_eq!(blob.get_slice_mut::<u32>(0, 2), &mut [1, 2]);
            blob.push(3u32);
            assert_eq!(blob.len(), 3);

            let removed = blob.remove::<u32>(1);
            assert_eq!(removed, 2);
            assert_eq!(blob.len(), 2);
            assert_eq!(blob.get::<u32>(0), &1);
            assert_eq!(blob.get::<u32>(1), &3);

            blob.push(4u32);
            assert_eq!(blob.len(), 3);
            assert_eq!(blob.get_slice::<u32>(0, 3), &[1, 3, 4]);
        }
    }

    #[test]
    fn test_blob_shrink() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None, 10);
        unsafe {
            blob.push(1);
            blob.push(2);
            blob.push(3);
        }

        assert_eq!(blob.len(), 3);
        assert_eq!(blob.capacity(), 10);

        blob.shrink_to(5);
        assert_eq!(blob.len(), 3);
        assert_eq!(blob.capacity(), 5);

        blob.shrink_to_fit();
        assert_eq!(blob.len(), 3);
        assert_eq!(blob.capacity(), 3);

        blob.shrink_to_fit_raw(2);
        assert_eq!(blob.len(), 2);
        assert_eq!(blob.capacity(), 2);

        blob.clear();
        assert_eq!(blob.len(), 0);
        assert_eq!(blob.capacity(), 2);

        blob.shrink_to_fit();
        assert_eq!(blob.capacity(), 0);

        unsafe { blob.push(1); }
        assert_eq!(blob.len(), 1);
        assert_eq!(blob.capacity(), 1);
        blob.reserve(1);
        assert_eq!(blob.len(), 1);
        assert_eq!(blob.capacity(), 2);
    }

    #[test]
    fn test_blob_zst() {
        let layout = Layout::new::<()>();
        let mut blob = BlobVec::new(layout, Some(|_| println!("dropping zst")), 2);
        assert_eq!(blob.len(), 0);
        assert_eq!(blob.capacity(), usize::MAX);

        unsafe {
            blob.push(());
            blob.push(());
            assert_eq!(blob.layout().size(), 0);
            assert_eq!(blob.layout().align(), 1);
            assert_eq!(blob.len(), 2);
            blob.clear();
            blob.push(());
            blob.reserve(1);
            assert_eq!(blob.len(), 1);
            assert_eq!(blob.capacity(), usize::MAX);
            blob.shrink_to_fit();
            blob.remove::<()>(0);
            assert_eq!(blob.len(), 0);
            assert_eq!(blob.capacity(), usize::MAX);
            assert_eq!(blob.layout().size(), 0);
            assert_eq!(blob.layout().align(), 1);
            blob.push(());
        };
    }

    #[test]
    fn test_blob_drop() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(
            layout,
            Some(|ptr| unsafe {
                let value = ptr.cast::<u32>().as_ref();
                println!("Dropping value: {}", value);
            }),
            0,
        );

        unsafe {
            blob.push(1);
            blob.push(2);
            blob.push(3);
            blob.clear();

            blob.push(100);
            blob.push(42);
            blob.push(200);
            let s = blob.remove::<u32>(1);
            println!("Removed value: {}", s);
            blob.shrink_to(0);
            blob.push(300);
            blob.push(400);
            blob.shrink_to_fit_raw(2);

            assert_eq!(blob.len(), 2);
            assert_eq!(blob.capacity(), 2);
        }

        // --nocapture should be
        // 1, 2, 3 - clear
        // then removed 42, so no drop
        // 300, 400 (shrink to fit)
        // 100, 200 - auto drop
    }
}

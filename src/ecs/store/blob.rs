use std::{
    alloc::{self, Layout},
    num::NonZero,
    ptr::NonNull,
};

type DropFn = unsafe fn(NonNull<u8>);

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
    pub fn new(layout: Layout, drop: Option<DropFn>, capacity: usize) -> Self {
        let data = Self::dangling(layout);

        let mut blob = Self {
            layout,
            data,
            len: 0,
            capacity: 0,
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
    pub fn reserve(&mut self, additional: usize) {
        let grow_by = (self.len + additional).saturating_sub(self.capacity);
        if grow_by > 0 {
            self.grow_by(grow_by);
        }
    }

    /// Reallocate the blob to a new capacity.
    unsafe fn reallocate(&mut self, new_capacity: usize) {
        let new_layout =
            layout_repeat(&self.layout, new_capacity).expect("Failed to repeat layout");
        let new_data = if self.capacity == 0 {
            alloc::alloc(new_layout)
        } else {
            let old_layout =
                layout_repeat(&self.layout, self.capacity).expect("Failed to repeat layout");
            alloc::realloc(self.data.as_ptr(), old_layout, new_layout.size())
        };

        self.data = NonNull::new(new_data).unwrap_or_else(|| alloc::handle_alloc_error(new_layout));

        self.capacity = new_capacity;
    }

    /// Grow the blob by `additional` elements.
    fn grow_by(&mut self, additional: usize) {
        let new_capacity = self
            .capacity
            .checked_add(additional)
            .expect("Overflow in capacity");

        if new_capacity > self.capacity {
            unsafe {
                self.reallocate(new_capacity);
            }
        }
    }

    fn copy_nonoverlapping(&self, src: NonNull<u8>, dst: NonNull<u8>) {
        unsafe {
            core::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), self.layout.size());
        }
    }

    fn swap_nonoverlapping(&self, x: NonNull<u8>, y: NonNull<u8>) {
        unsafe {
            core::ptr::swap_nonoverlapping(x.as_ptr(), y.as_ptr(), self.layout.size());
        }
    }

    /// Get a pointer to the element at index `i`.
    unsafe fn get_raw(&self, i: usize) -> NonNull<u8> {
        debug_assert!(i <= self.len, "Index out of bounds");
        self.get_raw_unchecked(i)
    }

    /// Get a pointer to the element at index `i` without bounds checking.
    unsafe fn get_raw_unchecked(&self, i: usize) -> NonNull<u8> {
        let offset = i * self.layout.size();
        let ptr = unsafe { self.data.as_ptr().add(offset) };
        NonNull::new_unchecked(ptr)
    }

    /// Push a new element to the blob.
    unsafe fn push_raw(&mut self, value: NonNull<u8>) {
        self.reserve(1);
        let dst = self.get_raw_unchecked(self.len);
        self.len += 1;
        self.copy_nonoverlapping(value, dst);
    }

    /// Swap remove the element at index `i`.
    unsafe fn swap_remove_raw(&mut self, i: usize) -> NonNull<u8> {
        debug_assert!(i <= self.len, "Index out of bounds");
        let last = self.len - 1;
        let last_ptr = self.get_raw(last);

        if i != last {
            let i_ptr = self.get_raw(i);
            self.swap_nonoverlapping(last_ptr, i_ptr);
        }

        self.len -= 1;
        last_ptr
    }

    /// Get a mutable slice of the blob.
    unsafe fn get_slice_raw(&self, start: usize, end: usize) -> &mut [u8] {
        debug_assert!(start < end, "Start index must be less than end index");
        debug_assert!(start <= self.len, "Start index out of bounds");
        debug_assert!(end <= self.len, "End index out of bounds");

        let start_ptr = self.get_raw(start);
        core::slice::from_raw_parts_mut(start_ptr.as_ptr(), end - start)
    }

    /// Shrink the blob to fit the given capacity.
    unsafe fn shrink_to_fit_raw(&mut self, cap: usize) {
        if cap >= self.capacity {
            return;
        }

        if cap < self.len {
            self.clear_range(cap, self.len);
            self.len = cap;
        }

        if cap == 0 {
            self.deallocate();
            self.data = Self::dangling(self.layout);
            self.capacity = 0;
        } else {
            self.reallocate(cap);
        }
    }

    /// Shrink the blob to fit the given capacity.
    /// Minimum capacity is the current length.
    pub fn shrink_to(&mut self, cap: usize) {
        let cap = self.len.max(cap);

        if cap < self.capacity {
            unsafe {
                self.shrink_to_fit_raw(cap);
            }
        }
    }

    /// Shrink the blob to fit the current length.
    pub fn shrink_to_fit(&mut self) {
        self.shrink_to(0);
    }

    /// Convert a value to a pointer.
    fn type_to_ptr<T>(value: T) -> NonNull<u8> {
        let ptr = Box::into_raw(Box::new(value)) as *mut u8;
        unsafe { NonNull::new_unchecked(ptr) }
    }

    /// Convert a pointer to a value.
    fn ptr_to_type<T>(ptr: NonNull<u8>) -> T {
        unsafe { ptr.cast::<T>().as_ptr().read() }
    }

    /// Push a new element to the blob.
    pub fn push<T>(&mut self, value: T) {
        let ptr = Self::type_to_ptr(value);
        unsafe {
            self.push_raw(ptr);
        }
    }

    /// Remove an element from the blob.
    pub fn remove<T>(&mut self, i: usize) -> T {
        let ptr = unsafe { self.swap_remove_raw(i) };
        Self::ptr_to_type(ptr)
    }

    /// Get a reference to an element
    pub fn get<T>(&self, i: usize) -> &T {
        let ptr = unsafe { self.get_raw(i) };
        unsafe { ptr.cast::<T>().as_ref() }
    }

    /// Get a mutable reference to an element
    pub fn get_mut<T>(&mut self, i: usize) -> &mut T {
        let ptr = unsafe { self.get_raw(i) };
        unsafe { ptr.cast::<T>().as_mut() }
    }

    /// Get a slice of the blob
    pub fn get_slice<T>(&self, start: usize, end: usize) -> &[T] {
        unsafe {
            let slice = self.get_slice_raw(start, end);
            core::mem::transmute(slice)
        }
    }

    /// Get a mutable slice of the blob
    pub fn get_slice_mut<T>(&mut self, start: usize, end: usize) -> &mut [T] {
        unsafe {
            let slice = self.get_slice_raw(start, end);
            core::mem::transmute(slice)
        }
    }

    /// Clear the blob
    pub fn clear(&mut self) {
        if self.len == 0 {
            return;
        }

        let len = self.len;
        self.len = 0;

        unsafe { self.clear_range(0, len) };
    }

    /// Clear a range of the blob, `start..end`
    unsafe fn clear_range(&mut self, start: usize, end: usize) {
        debug_assert!(start < end, "Start index must be less than end index");

        if let Some(drop_fn) = self.drop {
            for i in start..end {
                let ptr = self.get_raw_unchecked(i);
                drop_fn(ptr);
            }
        }
    }

    /// Deallocate the blob, does not call drop on the elements
    unsafe fn deallocate(&mut self) {
        let layout = layout_repeat(&self.layout, self.capacity).expect("Failed to repeat layout");

        if layout.size() != 0 {
            alloc::dealloc(self.data.as_ptr(), layout);
        }
    }
}

impl Drop for BlobVec {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            self.deallocate();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::Layout;

    #[test]
    fn test_blob_vec() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None, 2);
        assert_eq!(blob.len(), 0);
        assert_eq!(blob.capacity(), 2);
        assert_eq!(blob.layout().size(), 4);
        assert_eq!(blob.layout().align(), 4);
        assert_eq!(blob.layout().size(), std::mem::size_of::<u32>());

        blob.push(1u32);
        blob.push(2u32);

        assert_eq!(blob.len(), 2);
        assert_eq!(blob.get::<u32>(0), &1);
        assert_eq!(blob.get::<u32>(1), &2);
        assert_eq!(blob.get_slice::<u32>(0, 2), &[1, 2]);
        assert_eq!(blob.get_slice_mut::<u32>(0, 2), &mut [1, 2]);
        blob.push(3);
        assert_eq!(blob.len(), 3);

        let removed = blob.remove::<u32>(1);
        assert_eq!(removed, 2);
        assert_eq!(blob.len(), 2);
        assert_eq!(blob.get::<u32>(0), &1);
        assert_eq!(blob.get::<u32>(1), &3);

        blob.push(4);
        assert_eq!(blob.len(), 3);
        assert_eq!(blob.get_slice::<u32>(0, 3), &[1, 3, 4]);
    }

    #[test]
    fn test_blob_vec_shrink() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None, 10);
        blob.push(1);
        blob.push(2);
        blob.push(3);

        assert_eq!(blob.len(), 3);
        assert_eq!(blob.capacity(), 10);

        blob.shrink_to(5);
        assert_eq!(blob.len(), 3);
        assert_eq!(blob.capacity(), 5);

        blob.shrink_to_fit();
        assert_eq!(blob.len(), 3);
        assert_eq!(blob.capacity(), 3);

        unsafe { blob.shrink_to_fit_raw(2) };
        assert_eq!(blob.len(), 2);
        assert_eq!(blob.capacity(), 2);

        blob.clear();
        assert_eq!(blob.len(), 0);
        assert_eq!(blob.capacity(), 2);

        blob.shrink_to_fit();
        assert_eq!(blob.capacity(), 0);

        blob.push(1);
        assert_eq!(blob.len(), 1);
        assert_eq!(blob.capacity(), 1);
        blob.reserve(1);
        assert_eq!(blob.len(), 1);
        assert_eq!(blob.capacity(), 2);
    }

    #[test]
    fn test_blob_vec_drop() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, Some(|ptr| {
            unsafe {
                let value = ptr.cast::<u32>().as_ref();
                println!("Dropping value: {}", value);
            }
        }), 0);

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
        unsafe { blob.shrink_to_fit_raw(2) };

        assert_eq!(blob.len(), 2);
        assert_eq!(blob.capacity(), 2);

        // --nocapture should be
        // 1, 2, 3 - clear
        // then removed 42, so no drop
        // 300, 400 (shrink to fit)
        // 100, 200 - auto drop
    }
}

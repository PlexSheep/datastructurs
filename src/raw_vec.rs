use std::{
    alloc::{self, Layout},
    ptr::NonNull,
};

#[derive(Clone, Debug)]
pub(crate) struct RawVec<T> {
    pub(crate) ptr: NonNull<T>,
    pub(crate) capacity: usize,
}

impl<T> RawVec<T> {
    pub(crate) fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: 0,
        }
    }

    // See rustonomicon, chapter 9.2
    pub(crate) fn grow_by(&mut self, added_capacity: usize) {
        let new_cap = self.capacity + added_capacity;
        // `Layout::array` checks that the number of bytes is <= usize::MAX,
        // but this is redundant since old_layout.size() <= isize::MAX,
        // so the `unwrap` should never fail.
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        if new_layout.size() > isize::MAX as usize {
            alloc::handle_alloc_error(new_layout);
        }

        let new_ptr = if self.capacity == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.capacity).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // If allocation fails, `new_ptr` will be null, in which case we abort.
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.capacity = new_cap;
    }

    pub(crate) fn grow(&mut self) {
        if self.capacity == 0 {
            self.grow_by(1);
        } else {
            self.grow_by(self.capacity);
        }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            let layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

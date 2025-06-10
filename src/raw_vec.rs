use std::{
    alloc::{self, Layout},
    ptr::NonNull,
};

use crate::trace;

#[derive(Clone)]
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
        trace!(
            "growing raw_vec at {:?} from {} to {}",
            self.ptr, self.capacity, new_cap
        );
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
        // NOTE: We need to free the allocated memory here,
        // otherwise there definitely is a memory leak.
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

#[cfg(test)]
mod test {
    use std::ptr;

    use super::RawVec;

    #[test]
    fn test_rawvec_alloc_dealloc() {
        let s = 2_000;
        let mut v = RawVec::<u32>::new();
        v.grow_by(s);
        unsafe {
            ptr::write_bytes(v.ptr.as_ptr(), b'A', s);
        }
        drop(v)
    }
}

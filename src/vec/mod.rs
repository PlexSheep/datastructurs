//! Custom implementation of the Vector datastructure
//!
//! Many thanks to the rustonomicon, chapter 9:
//! https://doc.rust-lang.org/nomicon/vec/vec.html

use std::{
    mem,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr,
};

use impls::{Drain, RawIter};

use crate::raw_vec::RawVec;

mod impls;

#[derive(Clone, Debug)]
pub struct Vec<T> {
    used: usize,
    buf: RawVec<T>,
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        if mem::size_of::<T>() == 0 {
            panic!("We're not ready to handle ZSTs");
        }
        Vec {
            used: 0,
            buf: RawVec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if mem::size_of::<T>() == 0 {
            panic!("We're not ready to handle ZSTs");
        }

        let mut v = Self::new();
        v.reserve(capacity);
        v
    }

    pub fn from_slice(data: &[T]) -> Self {
        let mut v = Vec::<T>::with_capacity(data.len());
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), v.as_mut_ptr(), data.len());
        }
        v
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.used == 0 {
            None
        } else {
            self.used -= 1;
            unsafe { Some(ptr::read(self.buf.ptr.as_ptr().add(self.used))) }
        }
    }

    pub fn push(&mut self, value: T) {
        if self.used == self.buf.capacity {
            self.buf.grow();
        }

        unsafe {
            ptr::write(self.buf.ptr.as_ptr().add(self.used), value);
        }

        self.used += 1;
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.used
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.buf.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        // Note: `<=` because it's valid to insert after everything
        // which would be equivalent to push.
        assert!(index <= self.used, "index out of bounds");
        if self.used == self.buf.capacity {
            self.buf.grow();
        }

        unsafe {
            // ptr::copy(src, dest, len): "copy from src to dest len elems"
            ptr::copy(
                self.buf.ptr.as_ptr().add(index),
                self.buf.ptr.as_ptr().add(index + 1),
                self.used - index,
            );
            ptr::write(self.buf.ptr.as_ptr().add(index), elem);
        }

        self.used += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        // Note: `<` because it's *not* valid to remove after everything
        assert!(index < self.used, "index out of bounds");
        unsafe {
            self.used -= 1;
            let result = ptr::read(self.buf.ptr.as_ptr().add(index));
            ptr::copy(
                self.buf.ptr.as_ptr().add(index + 1),
                self.buf.ptr.as_ptr().add(index),
                self.used - index,
            );
            result
        }
    }

    pub fn reserve(&mut self, added_capacity: usize) {
        self.buf.grow_by(added_capacity);
    }

    #[must_use]
    pub fn split_off(&mut self, at: usize) -> Self {
        let other_len = self.used - at;
        let mut other = Self::with_capacity(other_len);
        unsafe {
            self.set_len(at);
            other.set_len(other_len);
            ptr::copy_nonoverlapping(self.as_ptr().add(at), other.as_mut_ptr(), other.len());
        }
        other
    }

    unsafe fn set_len(&mut self, new_length: usize) {
        self.used = new_length
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self.buf.ptr.as_ptr()
    }

    #[must_use]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    pub fn drain_all(&mut self) -> Drain<T> {
        let iter = unsafe { RawIter::new(self) };

        self.used = 0;

        Drain {
            iter,
            marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_vec_create() {
        let _v = Vec::<u64>::new();
    }

    #[test]
    fn test_vec_pushpop_num() {
        let mut v = Vec::new();
        let vals = &[19, 9, 14, 255, 19191919, 13890, 21521, 1251, 6216, 1830];

        for val in vals {
            v.push(*val);
        }
        for val in vals.iter().rev() {
            assert_eq!(v.pop().unwrap(), *val);
        }
    }

    #[test]
    fn test_vec_pushpop_str() {
        let mut v = Vec::new();
        let vals = &["AAAA", "ABBAB", "BBABBABBAJJJ"];

        for val in vals {
            v.push(*val);
        }
        for val in vals.iter().rev() {
            assert_eq!(v.pop().unwrap(), *val);
        }
    }

    #[test]
    fn test_vec_pushindex_num() {
        let mut v = Vec::new();
        let vals = &[19, 9, 14, 255, 19191919, 13890, 21521, 1251, 6216, 1830];

        for val in vals {
            v.push(*val);
        }
        for (idx, val) in vals.iter().enumerate() {
            assert_eq!(v[idx], *val);
        }
    }

    #[test]
    fn test_vec_pushindex_str() {
        let mut v = Vec::new();
        let vals = &["AAAA", "ABBAB", "BBABBABBAJJJ"];

        for val in vals {
            v.push(*val);
        }
        for (idx, val) in vals.iter().enumerate() {
            assert_eq!(v[idx], *val);
        }
    }
}

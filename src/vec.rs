//! Custom implementation of the Vector datastructure
//!
//! Many thanks to the rustonomicon, chapter 9:
//! https://doc.rust-lang.org/nomicon/vec/vec.html

use std::{
    mem,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr,
};

use crate::raw_vec::RawVec;

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
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T> IndexMut<usize> for Vec<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.buf.ptr.as_ptr(), self.used) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.buf.ptr.as_ptr(), self.used) }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

impl<T> From<&[T]> for Vec<T> {
    fn from(value: &[T]) -> Self {
        Self::from_slice(value)
    }
}

impl<T: PartialEq> PartialEq for Vec<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for i in 0..self.len() {
            if self[i] != other[i] {
                return false;
            }
        }
        true
    }
}

impl<T: Eq> Eq for Vec<T> {}

pub struct VecIntoIter<T> {
    vec: Vec<T>,
    index: usize,
}

impl<T> Iterator for VecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len() {
            let item = unsafe { std::ptr::read(self.vec.as_ptr().add(self.index)) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct VecRefIter<'a, T> {
    vec: &'a Vec<T>,
    index: usize,
}

impl<'a, T> Iterator for VecRefIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len() {
            let item = &self.vec[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = VecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        VecIntoIter {
            vec: self,
            index: 0,
        }
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = VecRefIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VecRefIter {
            vec: self,
            index: 0,
        }
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Vec::new();
        for item in iter {
            vec.push(item);
        }
        vec
    }
}

impl<T> Extend<T> for Vec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}
unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

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

use std::fmt::Debug;
use std::ops::{Index, IndexMut};

use super::*;

impl<A: IntrusiveListAccessor<T>, T> Drop for IntrusiveList<T, A> {
    fn drop(&mut self) {
        for item in self.iter_mut() {
            let node = A::get_node_mut(item);
            node.prev = None;
            node.next = None;
        }
    }
}

impl<A: IntrusiveListAccessor<T>, T> IndexMut<usize> for IntrusiveList<T, A> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
            .expect("No element with that index in the linked list")
    }
}

impl<A: IntrusiveListAccessor<T>, T> Index<usize> for IntrusiveList<T, A> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .expect("No element with that index in the linked list")
    }
}

impl Default for ListLink {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, A: IntrusiveListAccessor<T>> Default for IntrusiveList<T, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug, A: IntrusiveListAccessor<T>> Debug for IntrusiveList<T, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl Debug for ListLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("next", &self.next)
            .field("prev", &self.prev)
            .field("_addr", &self.as_ptr())
            .finish()
    }
}

pub struct Iter<'a, A: IntrusiveListAccessor<T>, T> {
    pub(crate) current: OpNodePtr,
    pub(crate) remaining: usize,
    pub(crate) _phantom: std::marker::PhantomData<(&'a T, A)>,
}

pub struct IterMut<'a, A: IntrusiveListAccessor<T>, T> {
    pub(crate) current: OpNodePtr,
    pub(crate) remaining: usize,
    pub(crate) _phantom: std::marker::PhantomData<(&'a mut T, A)>,
}

impl<'a, A: IntrusiveListAccessor<T>, T> Iterator for Iter<'a, A, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let current_ptr = self.current?;
        let current_node = deref_node(current_ptr);

        self.current = current_node.next;
        self.remaining -= 1;

        Some(unsafe { A::from_node(current_node) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, A: IntrusiveListAccessor<T>, T> Iterator for IterMut<'a, A, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let current_ptr = self.current?;
        let current_node = deref_node_mut(current_ptr);

        self.current = current_node.next;
        self.remaining -= 1;

        Some(unsafe { A::from_node_mut(current_node) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

use std::fmt::Debug;
use std::ops::{Index, IndexMut};

use super::{LinkedList, Node, OpNodePtr, deref_node, deref_node_mut};

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> IndexMut<usize> for LinkedList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
            .expect("No element with that index in the linked list")
    }
}

impl<T> Index<usize> for LinkedList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .expect("No element with that index in the linked list")
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("value", &self.value)
            .field("next", &self.next)
            .field("prev", &self.prev)
            .field("_addr", &self.as_ptr())
            .finish()
    }
}

pub struct Iter<'a, T> {
    pub(crate) current: OpNodePtr<T>,
    pub(crate) remaining: usize,
    pub(crate) _phantom: std::marker::PhantomData<&'a T>,
}

pub struct IterMut<'a, T> {
    pub(crate) current: OpNodePtr<T>,
    pub(crate) remaining: usize,
    pub(crate) _phantom: std::marker::PhantomData<&'a mut T>,
}

pub struct IntoIter<T> {
    pub(crate) list: LinkedList<T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let current_ptr = self.current?;
        let current_node = deref_node(current_ptr);

        self.current = current_node.next;
        self.remaining -= 1;

        Some(&current_node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let current_ptr = self.current?;
        let current_node = deref_node_mut(current_ptr);

        self.current = current_node.next;
        self.remaining -= 1;

        Some(&mut current_node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

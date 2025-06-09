use std::fmt::Debug;
use std::ops::{Index, IndexMut};

use super::{LinkedList, Node};

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
        f.debug_struct("LinkedList")
            .field("len", &self.len)
            .field("head", &self.head)
            .field("tail", &self.tail)
            .finish()
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

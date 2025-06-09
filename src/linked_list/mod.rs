use std::fmt::{Debug, Write};
use std::ptr::NonNull;

use impls::{Iter, IterMut};

mod impls;

pub(crate) type NodePtr<T> = NonNull<Node<T>>;
pub(crate) type OpNodePtr<T> = Option<NodePtr<T>>;

pub(crate) struct Node<T> {
    pub(crate) value: T,
    pub(crate) next: OpNodePtr<T>,
    pub(crate) prev: OpNodePtr<T>,
}

pub struct LinkedList<T> {
    head: OpNodePtr<T>,
    tail: OpNodePtr<T>,
    len: usize,
}

impl<T> Node<T> {
    #[inline]
    #[must_use]
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            next: None,
            prev: None,
        }
    }

    fn as_ptr(&self) -> NodePtr<T> {
        let a: *const Self = self;
        unsafe { NodePtr::new_unchecked(a as *mut Self) }
    }
}

impl<T> LinkedList<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a new node and returns its pointer, updating the list length
    fn create_node(&mut self, value: T) -> NodePtr<T> {
        let node = Box::new(Node::new(value));
        let node_ptr = node.as_ptr();
        Box::leak(node);
        self.len += 1;
        node_ptr
    }

    /// Removes a node and returns its value, updating the list length
    fn destroy_node(&mut self, node_ptr: NodePtr<T>) -> T {
        let node = deref_node_box(node_ptr);
        self.len -= 1;
        node.value
    }

    /// Finds the node at the given index
    fn find_node(&self, index: usize) -> Option<NodePtr<T>> {
        if index >= self.len {
            return None;
        }

        // PERF: Choose direction based on index
        if index < self.len / 2 {
            // Search from head
            let mut current_ptr = self.head?;
            for _ in 0..index {
                current_ptr = deref_node(current_ptr).next?;
            }
            Some(current_ptr)
        } else {
            // Search from tail (reverse direction)
            let mut current_ptr = self.tail?;
            for _ in 0..(self.len - 1 - index) {
                current_ptr = deref_node(current_ptr).prev?;
            }
            Some(current_ptr)
        }
    }

    fn link_as_only_node(&mut self, node_ptr: NodePtr<T>) {
        debug_assert!(self.head.is_none() && self.tail.is_none());
        self.head = Some(node_ptr);
        self.tail = Some(node_ptr);
    }

    pub fn push_back(&mut self, element: T) {
        let p_node = self.create_node(element);

        match self.tail {
            None => self.link_as_only_node(p_node),
            Some(p_old_tail) => {
                deref_node_mut(p_old_tail).next = Some(p_node);
                deref_node_mut(p_node).prev = Some(p_old_tail);
                self.tail = Some(p_node)
            }
        }
    }

    pub fn push_front(&mut self, element: T) {
        let p_node = self.create_node(element);

        match self.head {
            None => self.link_as_only_node(p_node),
            Some(p_old_head) => {
                deref_node_mut(p_node).next = Some(p_old_head);
                deref_node_mut(p_old_head).prev = Some(p_node);
                self.head = Some(p_node)
            }
        }
    }

    /// Unlinks the head node and updates head pointer
    fn unlink_head(&mut self) -> NodePtr<T> {
        let head_ptr = self.head.expect("Cannot unlink head from empty list");
        let head_node = deref_node(head_ptr);

        match head_node.next {
            Some(new_head_ptr) => {
                // Update new head's prev pointer
                deref_node_mut(new_head_ptr).prev = None;
                self.head = Some(new_head_ptr);
            }
            None => {
                // This was the only node
                self.head = None;
                self.tail = None;
            }
        }

        head_ptr
    }

    /// Unlinks the tail node and updates tail pointer
    fn unlink_tail(&mut self) -> NodePtr<T> {
        let tail_ptr = self.tail.expect("Cannot unlink tail from empty list");
        let tail_node = deref_node(tail_ptr);

        match tail_node.prev {
            Some(new_tail_ptr) => {
                // Update new tail's next pointer
                deref_node_mut(new_tail_ptr).next = None;
                self.tail = Some(new_tail_ptr);
            }
            None => {
                // This was the only node
                self.head = None;
                self.tail = None;
            }
        }

        tail_ptr
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        debug_assert!(self.head.is_some());

        let head_ptr = self.unlink_head();
        Some(self.destroy_node(head_ptr))
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        debug_assert!(self.head.is_some());

        let tail_ptr = self.unlink_tail();
        Some(self.destroy_node(tail_ptr))
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
        debug_assert!(self.is_empty());
    }

    #[must_use]
    pub(crate) fn last_node(&self) -> Option<&Node<T>> {
        self.tail.map(|ptr| deref_node(ptr))
    }

    #[must_use]
    pub(crate) fn first_node(&self) -> Option<&Node<T>> {
        self.head.map(|ptr| deref_node(ptr))
    }

    #[must_use]
    pub(crate) fn last_node_mut(&mut self) -> Option<&mut Node<T>> {
        self.tail.map(|ptr| deref_node_mut(ptr))
    }

    #[must_use]
    pub(crate) fn first_node_mut(&mut self) -> Option<&mut Node<T>> {
        self.head.map(|ptr| deref_node_mut(ptr))
    }

    #[must_use]
    pub fn last(&self) -> Option<&T> {
        self.last_node().map(|n| &n.value)
    }

    #[must_use]
    pub fn first(&self) -> Option<&T> {
        self.first_node().map(|n| &n.value)
    }

    #[must_use]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.last_node_mut().map(|n| &mut n.value)
    }

    #[must_use]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.first_node_mut().map(|n| &mut n.value)
    }

    #[must_use]
    pub(crate) fn get_node(&self, index: usize) -> Option<&Node<T>> {
        self.find_node(index).map(|p| deref_node(p))
    }

    #[must_use]
    pub(crate) fn get_node_mut(&mut self, index: usize) -> Option<&mut Node<T>> {
        self.find_node(index).map(|p| deref_node_mut(p))
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.get_node(index).map(|n| &n.value)
    }

    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.get_node_mut(index).map(|n| &mut n.value)
    }

    #[must_use]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            current: self.head,
            remaining: self.len,
            _phantom: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            current: self.head,
            remaining: self.len,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: PartialEq> LinkedList<T> {
    #[must_use]
    pub fn contains(&self, element: &T) -> bool {
        let mut current_node = deref_node(match self.head {
            Some(h) => h,
            None => {
                return false;
            }
        });

        loop {
            if current_node.value == *element {
                return true;
            }

            match current_node.next {
                None => break,
                Some(p_next) => {
                    if p_next == self.head.unwrap() {
                        // reached end
                        break;
                    };
                    current_node = deref_node(p_next)
                }
            }
        }
        false
    }
}

impl<T: Debug> LinkedList<T> {
    pub fn format_node_content(&self) -> String {
        let mut buf = "Contents of LinkedList:\n".to_string();
        let mut current_node = deref_node(match self.head {
            Some(h) => h,
            None => {
                return "(No head)".to_string();
            }
        });

        loop {
            writeln!(&mut buf, "{current_node:?}").unwrap();

            match current_node.next {
                None => break,
                Some(p_next) => {
                    if p_next == self.head.unwrap() {
                        // reached end
                        break;
                    };
                    current_node = deref_node(p_next)
                }
            }
        }
        buf
    }
}

#[inline]
#[must_use]
fn deref_node_box<'a, T: 'a>(p: NodePtr<T>) -> Box<Node<T>> {
    unsafe { Box::from_raw(p.as_ptr()) }
}

#[inline]
#[must_use]
fn deref_node<'a, T: 'a>(p: NodePtr<T>) -> &'a Node<T> {
    unsafe { &*p.as_ptr() }
}

#[inline]
#[must_use]
#[allow(clippy::mut_from_ref)]
fn deref_node_mut<'a, T: 'a>(p: NodePtr<T>) -> &'a mut Node<T> {
    unsafe { &mut *p.as_ptr() }
}

#[cfg(test)]
mod tests;

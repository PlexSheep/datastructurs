use std::fmt::{Debug, Write};
use std::marker::PhantomPinned;
use std::{marker::PhantomData, ptr::NonNull};

use impls::{Iter, IterMut};

mod impls;

pub use datastructurs_macros::IntoIntrusiveList;

use crate::stable_ref::{StableRef, StableRefMut};
use crate::trace;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct ListLink {
    next: OpNodePtr,
    prev: OpNodePtr,
    // This structure must never move in memory
    _pin: PhantomPinned,
}

pub type ItemMut<'a, T> = StableRefMut<'a, T>;
pub type ItemRef<'a, T> = StableRef<'a, T>;
pub(crate) type NodePtr = NonNull<ListLink>;
pub(crate) type OpNodePtr = Option<NodePtr>;

pub trait IntrusiveListAccessor<T> {
    fn get_node(item: &T) -> &ListLink;
    fn get_node_mut(item: &mut T) -> &mut ListLink;
    unsafe fn from_node(node: &ListLink) -> &T;
    unsafe fn from_node_mut(node: &mut ListLink) -> &mut T;
}

// BUG: the references to the list elements are broken when the datastructure owning the data is
// reallocated. THis can happen when a Vec grows beyond it's capacity.
pub struct IntrusiveList<T, A: IntrusiveListAccessor<T>> {
    // TODO: make these private again
    pub head: OpNodePtr,
    pub tail: OpNodePtr,
    len: usize,
    marker: PhantomData<(T, A)>,
}

impl ListLink {
    pub const fn new() -> Self {
        Self {
            next: None,
            prev: None,
            _pin: PhantomPinned,
        }
    }

    fn as_ptr(&self) -> NodePtr {
        let a: *const Self = self;
        unsafe { NodePtr::new_unchecked(a as *mut Self) }
    }

    pub fn is_linked(&self) -> bool {
        self.next.is_some() || self.prev.is_some()
    }
}

impl<T, A: IntrusiveListAccessor<T>> IntrusiveList<T, A> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            marker: PhantomData,
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

    fn link_as_only_node(&mut self, p_node: NodePtr) {
        debug_assert!(self.head.is_none() && self.tail.is_none());
        self.head = Some(p_node);
        self.tail = Some(p_node);
    }

    pub fn push_back<'a>(&'a mut self, element: impl Into<ItemMut<'a, T>>) {
        let mut element: ItemMut<T> = element.into();
        let node = A::get_node_mut(element.as_mut());

        match self.tail {
            None => self.link_as_only_node(node.as_ptr()),
            Some(p_old_tail) => {
                deref_node_mut(p_old_tail).next = Some(node.as_ptr());
                node.prev = Some(p_old_tail);
                self.tail = Some(node.as_ptr())
            }
        }
        self.len += 1;
    }

    pub fn push_front<'a>(&'a mut self, element: impl Into<ItemMut<'a, T>>) {
        let mut element: ItemMut<T> = element.into();
        let node = A::get_node_mut(element.as_mut());

        match self.head {
            None => self.link_as_only_node(node.as_ptr()),
            Some(p_old_head) => {
                node.next = Some(p_old_head);
                deref_node_mut(p_old_head).prev = Some(node.as_ptr());
                self.head = Some(node.as_ptr())
            }
        }
        self.len += 1;
    }

    /// Finds the node at the given index
    fn find_node(&self, index: usize) -> OpNodePtr {
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

    pub fn remove<'a>(&'a mut self, item: &'a mut T) -> Option<&'a mut T> {
        let node = A::get_node_mut(item);
        if !node.is_linked() && !self.head.is_some_and(|head| deref_node(head) == node) {
            trace!("A completely foreign node was passed into remove!");
            return None;
        }

        // BUG: sometimes, head is set to none even though the element being removed is not the
        // last one
        if let Some(mut prev) = node.prev {
            unsafe { prev.as_mut().next = node.next }
        } else if self.head == Some(node.as_ptr()) {
            self.head = node.next
        }

        if let Some(mut next) = node.next {
            unsafe { next.as_mut().prev = node.prev }
        } else if self.tail == Some(node.as_ptr()) {
            self.tail = node.prev
        }

        node.prev = None;
        node.next = None;

        #[cfg(debug_assertions)]
        {
            debug_assert!(self.len > 0);
            debug_assert!(self.head.is_some());
            debug_assert!(self.tail.is_some());
        }

        self.len -= 1;
        Some(item)
    }

    pub fn pop_front(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }
        let item = unsafe {
            A::from_node_mut(deref_node_mut(
                self.head.expect("list not empty but head is none"),
            ))
        };
        self.remove(item)
    }

    pub fn pop_back(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }
        let item = unsafe {
            A::from_node_mut(deref_node_mut(
                self.tail.expect("list not empty but tail is none"),
            ))
        };
        self.remove(item)
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
        debug_assert!(self.is_empty());
    }

    pub fn front(&self) -> Option<&T> {
        Some(unsafe { A::from_node(self.head?.as_ref()) })
    }

    pub fn front_mut(&self) -> Option<&mut T> {
        Some(unsafe { A::from_node_mut(self.head?.as_mut()) })
    }

    pub fn back(&self) -> Option<&T> {
        Some(unsafe { A::from_node(self.tail?.as_ref()) })
    }

    pub fn back_mut(&self) -> Option<&mut T> {
        Some(unsafe { A::from_node_mut(self.tail?.as_mut()) })
    }

    #[must_use]
    pub(crate) fn get_node(&self, index: usize) -> Option<&ListLink> {
        self.find_node(index).map(deref_node)
    }

    #[must_use]
    pub(crate) fn get_node_mut(&mut self, index: usize) -> Option<&mut ListLink> {
        self.find_node(index).map(deref_node_mut)
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.get_node(index).map(|n| unsafe { A::from_node(n) })
    }

    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.get_node_mut(index)
            .map(|n| unsafe { A::from_node_mut(n) })
    }

    #[must_use]
    pub fn iter(&self) -> Iter<A, T> {
        Iter {
            current: self.head,
            remaining: self.len,
            _phantom: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<A, T> {
        IterMut {
            current: self.head,
            remaining: self.len,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<A: IntrusiveListAccessor<T>, T: PartialEq> IntrusiveList<T, A> {
    #[must_use]
    pub fn contains(&self, element: &T) -> bool {
        let mut current_node = deref_node(match self.head {
            Some(h) => h,
            None => {
                return false;
            }
        });

        loop {
            if *unsafe { A::from_node(current_node) } == *element {
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

impl<T: Debug, A: IntrusiveListAccessor<T>> IntrusiveList<T, A> {
    pub fn debug_nodes(&self) -> String {
        let mut buf = "LinkedList: {\n".to_string();
        let mut current_node = deref_node(match self.head {
            Some(h) => h,
            None => {
                return "(No head)".to_string();
            }
        });

        loop {
            writeln!(&mut buf, "\t{}", self.debug_node(current_node)).unwrap();

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
        write!(&mut buf, "}}").unwrap();
        buf
    }

    pub fn debug_node(&self, node: &ListLink) -> String {
        let mut buf = "Node: {".to_string();
        write!(&mut buf, " {:?} }}", unsafe { A::from_node(node) }).unwrap();
        buf
    }
}

#[inline]
#[must_use]
fn deref_node<'a>(p: NodePtr) -> &'a ListLink {
    unsafe { &*p.as_ptr() }
}

#[inline]
#[must_use]
#[allow(clippy::mut_from_ref)]
fn deref_node_mut<'a>(p: NodePtr) -> &'a mut ListLink {
    unsafe { &mut *p.as_ptr() }
}

// tests are external to make use of the proc macro

mod map;
mod set;
use std::ptr::NonNull;

use crate::vec::Vec;

pub use map::BTreeMap;
pub use set::BTreeSet;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Node<T: Ord> {
    keys: Vec<T>,
    parent: Option<NodePtr<T>>,
    children: Vec<NodePtr<T>>,
}

pub(crate) type NodePtr<T> = NonNull<Node<T>>;
pub(crate) type OpNodePtr<T> = Option<NodePtr<T>>;

pub const DEFAULT_BRANCH_FACTOR: usize = 100;

impl<T: Ord> Node<T> {
    fn store_on_heap(self) -> NodePtr<T> {
        unsafe { NodePtr::new_unchecked(Box::into_raw(Box::new(self))) }
    }

    fn as_ptr(&self) -> NodePtr<T> {
        let a: *const Self = self;
        unsafe { NodePtr::new_unchecked(a as *mut Self) }
    }

    fn drop(node_ptr: NodePtr<T>) {
        unsafe { drop(Box::from_raw(node_ptr.as_ptr())) }
    }
}

#[inline]
#[must_use]
fn deref_node<'a, T: Ord + 'a>(p: NodePtr<T>) -> &'a Node<T> {
    unsafe { &*p.as_ptr() }
}

#[inline]
#[must_use]
#[allow(clippy::mut_from_ref)]
fn deref_node_mut<'a, T: Ord + 'a>(p: NodePtr<T>) -> &'a mut Node<T> {
    unsafe { &mut *p.as_ptr() }
}

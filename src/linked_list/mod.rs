use std::ptr::NonNull;

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

    fn drop(node_ptr: NodePtr<T>) {
        unsafe { drop(Box::from_raw(node_ptr.as_ptr())) }
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

    pub fn push_back(&mut self, element: T) {
        let mut node = Box::new(Node::new(element));

        if let Some(p_last_node) = self.tail {
            let last_node = deref_node_mut(p_last_node);

            node.prev = Some(p_last_node);
            node.next = Some(self.head.unwrap());
            last_node.next = Some(node.as_ptr());
        } else {
            // node stays without connections
        }
        self.tail = Some(node.as_ptr());
        if self.head.is_none() {
            self.head = Some(node.as_ptr());
        }

        Box::leak(node); // we restore the box in the Drop implementation to free the memory

        self.len += 1;
    }

    pub fn push_front(&mut self, element: T) {
        let mut node = Box::new(Node::new(element));

        if let Some(p_first_node) = self.head {
            let first_node = deref_node_mut(p_first_node);

            node.next = Some(p_first_node);
            if let Some(first_node_prev) = first_node.prev {
                node.prev = Some(first_node_prev);
                first_node.prev = None;
            }
            first_node.prev = Some(node.as_ptr())
        } else {
            // node stays without connections
        }
        self.head = Some(node.as_ptr());
        if self.tail.is_none() {
            self.tail = Some(node.as_ptr());
        }

        Box::leak(node); // we restore the box in the Drop implementation to free the memory

        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        todo!()
    }

    pub fn pop_front(&mut self) -> Option<T> {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }

    #[must_use]
    pub(crate) fn last_node(&self) -> Option<&Node<T>> {
        todo!()
    }

    #[must_use]
    pub(crate) fn first_node(&self) -> Option<&Node<T>> {
        todo!()
    }

    #[must_use]
    pub(crate) fn last_node_mut(&mut self) -> Option<&mut Node<T>> {
        todo!()
    }

    #[must_use]
    pub(crate) fn first_node_mut(&mut self) -> Option<&mut Node<T>> {
        todo!()
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
        todo!()
    }

    #[must_use]
    pub(crate) fn get_node_mut(&mut self, index: usize) -> Option<&mut Node<T>> {
        todo!()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.get_node(index).map(|n| &n.value)
    }

    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.get_node_mut(index).map(|n| &mut n.value)
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

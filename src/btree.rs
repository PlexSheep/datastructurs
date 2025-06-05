use std::{marker::PhantomData, mem, ptr::NonNull};

use crate::vec::Vec;

pub const DEFAULT_DEGREE: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Node<T: Ord> {
    keys: Vec<T>,
    parent: Option<NodePtr<T>>,
    children: Vec<NodePtr<T>>,
}

type NodePtr<T> = NonNull<Node<T>>;
type OpNodePtr<T> = Option<NodePtr<T>>;

#[derive(Clone, Debug)]
pub struct BTree<T: Ord> {
    root: NodePtr<T>,
    props: BTreeProperties,
}

#[derive(Clone, Debug, Copy)]
pub struct BTreeProperties {
    degree: usize,
    max_keys: usize,
    min_keys: usize,
    mid_key_index: usize,
    len: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum BTreeError {}

impl<T: Ord> Node<T> {
    fn store_on_heap(self) -> NodePtr<T> {
        unsafe { NodePtr::new_unchecked(Box::into_raw(Box::new(self))) }
    }

    fn as_ptr(&mut self) -> NodePtr<T> {
        let a: *mut Self = self;
        unsafe { NodePtr::new_unchecked(a) }
    }

    fn child_ptr(&self, idx: usize) -> NodePtr<T> {
        self.children[idx]
    }

    fn parent_ptr(&self) -> OpNodePtr<T> {
        self.parent
    }

    fn drop(node_ptr: NodePtr<T>) {
        unsafe { drop(Box::from_raw(node_ptr.as_ptr())) }
    }
}

impl BTreeProperties {
    #[must_use]
    fn new(degree: usize) -> Self {
        assert!(degree >= 3, "B-tree degree must be at least 3");
        Self {
            degree,
            max_keys: degree - 1,
            min_keys: degree / 2,
            mid_key_index: (degree - 1) / 2,
            len: 0,
        }
    }

    fn split_child<T: Ord + Clone>(&self, parent_ptr: NodePtr<T>, child_index: usize) {
        let parent = deref_node_mut(&parent_ptr);
        let child = deref_node_mut(&parent.children[child_index]);

        let right_keys = child.keys.split_off(self.mid_key_index + 1);
        let middle_key = child.keys.pop().unwrap(); // We reinsert later

        let right_children = if !child.is_leaf() {
            Some(child.children.split_off(self.mid_key_index + 1))
        } else {
            None
        };

        let new_child_node =
            Node::new_with_data(self.degree, right_keys, right_children, Some(parent_ptr))
                .store_on_heap();

        parent.keys.insert(child_index, middle_key);
        parent.children.insert(child_index + 1, new_child_node);
    }

    #[must_use]
    fn is_full<T: Ord>(&self, node: &NodePtr<T>) -> bool {
        deref_node_ref(node).keys.len() >= self.max_keys
    }

    #[must_use]
    fn find_insertion_index<T: Ord>(keys: &[T], key: &T) -> usize {
        match keys.binary_search(key) {
            Ok(idx) | Err(idx) => idx,
        }
    }

    fn insert_non_full<T: Ord + Clone>(&self, node_ptr: NodePtr<T>, key: T) {
        let node = deref_node_mut(&node_ptr);
        let index = Self::find_insertion_index(&node.keys, &key);

        if node.is_leaf() {
            node.keys.insert(index, key);
        } else if self.is_full(&node.children[index]) {
            self.split_child(node_ptr, index);
            // After split, determine which child to recurse into
            let final_index = if index < node.keys.len() && node.keys[index] < key {
                index + 1
            } else {
                index
            };
            self.insert_non_full(node.children[final_index], key);
        } else {
            self.insert_non_full(node.children[index], key);
        }
    }
}

impl<T: Ord> Node<T> {
    #[must_use]
    fn new(degree: usize, parent: OpNodePtr<T>) -> Self {
        Node {
            keys: Vec::with_capacity(degree - 1),
            parent,
            children: Vec::with_capacity(degree),
        }
    }

    #[must_use]
    fn new_with_data(
        degree: usize,
        keys: Vec<T>,
        children: Option<Vec<NodePtr<T>>>,
        parent: OpNodePtr<T>,
    ) -> Self {
        Self {
            keys,
            parent,
            children: children.unwrap_or_else(|| Vec::with_capacity(degree)),
        }
    }

    #[must_use]
    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

impl<T: Ord + Clone> BTree<T> {
    pub fn new(branch_factor: usize) -> Self {
        let degree = 2 * branch_factor;
        Self {
            root: Node::new(degree, None).store_on_heap(),
            props: BTreeProperties::new(degree),
        }
    }

    pub fn clear(&mut self) {
        Node::drop(self.root);
        *self = Self::new(self.props.degree * 2)
    }

    pub fn insert(&mut self, key: T) {
        if self.props.is_full(&self.root) {
            // Create new root and make old root its child
            let new_root = Node::new(self.props.degree, None);
            let old_root = mem::replace(deref_node_mut(&self.root), new_root);
            deref_node_mut(&self.root)
                .children
                .push(old_root.store_on_heap());
            self.props.split_child(self.root, 0);
        }
        self.props.insert_non_full(self.root, key);
        self.props.len += 1;
    }

    #[must_use]
    pub fn contains(&self, key: &T) -> bool {
        let mut current = deref_node_ref(&self.root);
        loop {
            match current.keys.binary_search(key) {
                Ok(_) => return true,
                Err(idx) => {
                    if current.is_leaf() {
                        return false;
                    }
                    current = deref_node_ref(&current.children[idx]);
                }
            }
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        deref_node_ref(&self.root).keys.is_empty()
    }

    #[must_use]
    pub fn height(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let mut height = 1;
        let mut current = deref_node_ref(&self.root);
        while !current.is_leaf() {
            height += 1;
            current = deref_node_ref(&current.children[0]);
        }
        height
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.props.len
    }

    #[must_use]
    pub fn first(&self) -> Option<&T> {
        todo!()
    }

    #[must_use]
    pub fn last(&self) -> Option<&T> {
        todo!()
    }

    pub fn pop_first(&mut self) -> Option<T> {
        todo!()
    }

    pub fn pop_last(&mut self) -> Option<T> {
        todo!()
    }

    pub fn remove(&mut self, key: &T) -> Option<T> {
        todo!("remove")
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        todo!()
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        todo!()
    }

    #[must_use]
    pub fn memory_usage(&self) -> usize {
        todo!()
    }

    #[must_use]
    pub fn load_factor(&self) -> f64 {
        todo!()
    }

    // Iterator support - returns keys in sorted order
    #[must_use]
    pub fn iter(&self) -> BTreeIter<T> {
        BTreeIter::new(&self.root)
    }

    // Range query support
    #[must_use]
    pub fn range<'a>(&'a self, start: &T, end: &T) -> impl Iterator<Item = &'a T> {
        self.iter()
            .skip_while(move |&k| k < start)
            .take_while(move |&k| k <= end)
    }
}

impl<T: Ord> Drop for Node<T> {
    fn drop(&mut self) {
        for child_ptr in &self.children {
            Node::drop(*child_ptr);
        }
    }
}

impl<T: Ord> Drop for BTree<T> {
    fn drop(&mut self) {
        Node::drop(self.root);
    }
}

// Simple iterator implementation
pub struct BTreeIter<'a, T: Ord> {
    stack: Vec<(NodePtr<T>, usize)>,
    marker: PhantomData<&'a ()>,
}

impl<'a, T: Ord> BTreeIter<'a, T> {
    fn new(root_ptr: &'a NodePtr<T>) -> Self {
        let mut iter = BTreeIter {
            stack: Vec::new(),
            marker: PhantomData,
        };
        iter.push_left_path(root_ptr, 0);
        iter
    }

    fn push_left_path(&mut self, node_ptr: &'a NodePtr<T>, start_idx: usize) {
        let mut node = deref_node_mut(node_ptr);
        loop {
            self.stack.push((node.as_ptr(), start_idx));
            if node.is_leaf() {
                break;
            }
            node = deref_node_mut(&node.children[start_idx]);
        }
    }
}

impl<'a, T: Ord + 'a> Iterator for BTreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node_ptr, idx)) = self.stack.pop() {
            let node = deref_node(node_ptr);
            if idx < node.keys.len() {
                let key = &node.keys[idx];

                // Prepare for next iteration
                if !node.is_leaf() && idx + 1 < node.children.len() {
                    self.push_left_path(&node.children[idx + 1], 0);
                }

                // Push current node back with incremented index
                if idx + 1 < node.keys.len() {
                    self.stack.push((node_ptr, idx + 1));
                }

                return Some(key);
            }
        }
        None
    }
}

#[inline]
#[must_use]
fn deref_node<'a, T: Ord + 'a>(p: NodePtr<T>) -> &'a Node<T> {
    unsafe { &*p.as_ptr() }
}

#[inline]
#[must_use]
fn deref_node_ref<T: Ord>(p: &NodePtr<T>) -> &Node<T> {
    unsafe { &*p.as_ptr() }
}

#[inline]
#[must_use]
#[allow(clippy::mut_from_ref)]
fn deref_node_mut<T: Ord>(p: &NodePtr<T>) -> &mut Node<T> {
    unsafe { &mut *p.as_ptr() }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_btree_create() {
        let _tree = BTree::<u32>::new(DEFAULT_DEGREE);
    }

    #[test]
    fn test_btree_insert_contains_remove_in_order() {
        let mut tree = BTree::<u32>::new(3); // Small degree for easier testing
        let data = &[10, 20, 5, 6, 12, 30, 7, 17];

        for &value in data {
            tree.insert(value);
        }

        for &value in data {
            assert!(tree.contains(&value), "Tree should contain {}", value);
        }

        assert!(!tree.contains(&999), "Tree should not contain 999");

        for &value in data {
            tree.remove(&value);
        }

        assert!(tree.is_empty())
    }

    #[test]
    fn test_btree_insert_contains_remove_out_of_order() {
        let mut tree = BTree::<u32>::new(3); // Small degree for easier testing
        let data = &[10, 20, 5, 6, 12, 30, 7, 17];

        for &value in data {
            tree.insert(value);
        }

        for &value in data.iter().step_by(2) {
            tree.remove(&value);
        }

        for &value in data.iter().skip(1).step_by(2) {
            assert!(tree.contains(&value), "Tree should contain {}", value);
        }

        for &value in data.iter().step_by(2) {
            assert!(!tree.contains(&value), "Tree should not contain {}", value);
        }

        assert!(!tree.contains(&999), "Tree should not contain 999");

        for &value in data.iter().skip(1).step_by(2) {
            tree.remove(&value);
        }

        assert!(tree.is_empty())
    }

    #[test]
    fn test_btree_iteration() {
        let mut tree = BTree::new(3);
        let data = [10, 20, 5, 6, 12, 30, 7, 17];

        for value in data {
            tree.insert(value);
        }

        let mut sorted = Vec::new();
        for &value in tree.iter() {
            sorted.push(value);
        }

        let expected = [5, 6, 7, 10, 12, 17, 20, 30];
        assert_eq!(sorted.len(), expected.len());
        for i in 0..sorted.len() {
            assert_eq!(sorted[i], expected[i]);
        }
    }

    #[test]
    fn test_btree_height() {
        let mut tree = BTree::new(3);
        assert_eq!(tree.height(), 0);

        tree.insert(1);
        assert_eq!(tree.height(), 1);

        // Insert enough to force splits
        for i in 2..=10 {
            tree.insert(i);
        }
        assert!(tree.height() > 1);
    }

    #[test]
    fn test_btree_moderate_dataset() {
        let mut tree = BTree::<u32>::new(50);
        let mut data = Vec::new();
        for i in 0..10000 {
            data.push(i);
        }

        for i in 0..data.len() {
            tree.insert(data[i]);
        }

        // Verify first 100 samples
        for i in 0..100 {
            assert!(tree.contains(&data[i]));
        }
    }

    #[test]
    fn test_btree_iter() {
        let data: Vec<_> = (0..9999).collect();
        let mut tree = BTree::new(DEFAULT_DEGREE);
        for d in &data {
            tree.insert(d);
        }

        for key in tree.iter() {
            assert!(data.contains(key))
        }
    }

    #[test]
    #[ignore = "still WIP"]
    fn test_btree_stress() {
        let mut tree = BTree::new(DEFAULT_DEGREE);
        let range = 0..5_000_000;
        for d in range.clone() {
            tree.insert(d);
        }
        println!("Tree height: {}", tree.height());
        println!("Tree len: {}", tree.len());

        for key in range.clone() {
            assert!(tree.contains(&key))
        }

        for key in range.into_iter().rev() {
            assert_eq!(tree.pop_last().unwrap(), key)
        }
        assert!(tree.is_empty())
    }
}

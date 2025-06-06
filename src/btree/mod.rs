use std::{fmt::Debug, mem, ptr::NonNull};

use impls::BTreeIter;

use crate::vec::Vec;

mod impls;

pub const DEFAULT_BRANCH_FACTOR: usize = 100;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Node<T: Ord> {
    keys: Vec<T>,
    parent: Option<NodePtr<T>>,
    children: Vec<NodePtr<T>>,
}

pub(crate) type NodePtr<T> = NonNull<Node<T>>;
pub(crate) type OpNodePtr<T> = Option<NodePtr<T>>;

#[derive(Clone)]
pub struct BTree<T: Ord + Clone> {
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

    fn split_child<T: Ord>(&self, parent_ptr: NodePtr<T>, child_index: usize) {
        let parent = deref_node_mut(parent_ptr);
        let child = deref_node_mut(parent.children[child_index]);

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
        deref_node(*node).keys.len() >= self.max_keys
    }

    #[must_use]
    fn find_insertion_index<T: Ord>(keys: &[T], key: &T) -> usize {
        match keys.binary_search(key) {
            Ok(idx) | Err(idx) => idx,
        }
    }

    fn insert_non_full<T: Ord>(&self, node_ptr: NodePtr<T>, key: T) {
        let node = deref_node_mut(node_ptr);
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
        // NOTE: seems like this calls the Drop impl of the old tree too,
        // analysis with vanguard shows no memory leaks here.
        *self = Self::new(self.props.degree * 2)
    }

    pub fn insert(&mut self, key: T) {
        if self.props.is_full(&self.root) {
            // Create new root and make old root its child
            let new_root = Node::new(self.props.degree, None);
            let mut old_root = mem::replace(deref_node_mut(self.root), new_root);
            old_root.parent = Some(self.root);
            deref_node_mut(self.root)
                .children
                .push(old_root.store_on_heap());
            self.props.split_child(self.root, 0);
        }
        self.props.insert_non_full(self.root, key);
        self.props.len += 1;
    }

    #[must_use]
    pub fn contains(&self, key: &T) -> bool {
        let mut current = deref_node(self.root);
        loop {
            match current.keys.binary_search(key) {
                Ok(_) => return true,
                Err(idx) => {
                    if current.is_leaf() {
                        return false;
                    }
                    current = deref_node(current.children[idx]);
                }
            }
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        deref_node(self.root).keys.is_empty()
    }

    #[must_use]
    pub fn height(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let mut height = 1;
        let mut current = deref_node(self.root);
        while !current.is_leaf() {
            height += 1;
            current = deref_node(current.children[0]);
        }
        height
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.props.len
    }

    #[must_use]
    pub fn first(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        let mut current = deref_node(self.root);
        loop {
            if current.is_leaf() {
                return Some(current.keys.first().unwrap());
            } else {
                current = deref_node(*current.children.first().unwrap());
            }
        }
    }

    #[must_use]
    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        let mut current = deref_node(self.root);
        loop {
            if current.is_leaf() {
                return Some(current.keys.last().unwrap());
            } else {
                current = deref_node(*current.children.last().unwrap());
            }
        }
    }

    pub fn pop_first(&mut self) -> Option<T> {
        self.remove(&self.first().cloned()?)
    }

    pub fn pop_last(&mut self) -> Option<T> {
        self.remove(&self.last().cloned()?)
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        let mut depth = 0;
        let mut current = deref_node(self.root);
        loop {
            depth += 1;
            if current.is_leaf() {
                return depth;
            } else {
                current = deref_node(current.children[0])
            }
        }
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        Self::node_count_inner(self.root)
    }

    fn node_count_inner(node_ptr: NodePtr<T>) -> usize {
        let node = deref_node(node_ptr);
        let mut total = 1;
        for child_ptr in &node.children {
            total += Self::node_count_inner(*child_ptr);
        }
        total
    }

    #[must_use]
    pub fn iter(&self) -> BTreeIter<T> {
        BTreeIter::new(&self.root)
    }

    pub fn range<'a>(&'a self, start: &T, end: &T) -> impl Iterator<Item = &'a T> {
        self.iter()
            .skip_while(move |&k| k < start)
            .take_while(move |&k| k <= end)
    }
}

// removing keys
// this is so fucking complicated
//
// To be honest, this whole impl block is LLM generated.
impl<T: Ord + Clone> BTree<T> {
    pub fn remove(&mut self, key: &T) -> Option<T> {
        let result = self.remove_from_node(self.root, key);

        // Handle root underflow - if root is empty but has children, promote the only child
        let root_node = deref_node(self.root);
        if root_node.keys.is_empty() && !root_node.children.is_empty() {
            let old_root = self.root;
            self.root = root_node.children[0];

            // Update the new root's parent to None
            deref_node_mut(self.root).parent = None;

            // Prevent the old root from dropping its children
            deref_node_mut(old_root).children.clear();
            Node::drop(old_root);
        }

        if result.is_some() {
            self.props.len -= 1;
        }
        result
    }

    fn remove_from_node(&mut self, node_ptr: NodePtr<T>, key: &T) -> Option<T> {
        let node = deref_node_mut(node_ptr);

        match node.keys.binary_search(key) {
            Ok(idx) => {
                // Key found in this node
                if node.is_leaf() {
                    // Case 1: Key is in a leaf node - simply remove it
                    node.keys.remove(idx)
                } else {
                    // Case 2: Key is in an internal node
                    self.remove_from_internal_node(node_ptr, idx)
                }
            }
            Err(idx) => {
                // Key not in this node
                if node.is_leaf() {
                    // Key doesn't exist in the tree
                    None
                } else {
                    // Recurse to the appropriate child
                    let child_ptr = node.children[idx];

                    // Ensure the child has enough keys before recursing
                    if deref_node(child_ptr).keys.len() <= self.props.min_keys {
                        self.ensure_child_has_enough_keys(node_ptr, idx);

                        // After rebalancing, we need to search again as indices may have changed
                        let node = deref_node(node_ptr);
                        let new_idx = match node.keys.binary_search(key) {
                            Ok(i) => {
                                // Key moved up to this node
                                return if node.is_leaf() {
                                    deref_node_mut(node_ptr).keys.remove(i)
                                } else {
                                    self.remove_from_internal_node(node_ptr, i)
                                };
                            }
                            Err(i) => i,
                        };

                        self.remove_from_node(node.children[new_idx], key)
                    } else {
                        self.remove_from_node(child_ptr, key)
                    }
                }
            }
        }
    }

    fn remove_from_internal_node(&mut self, node_ptr: NodePtr<T>, key_idx: usize) -> Option<T> {
        let node = deref_node(node_ptr);
        let key = node.keys[key_idx].clone();

        let left_child = node.children[key_idx];
        let right_child = node.children[key_idx + 1];

        if deref_node(left_child).keys.len() > self.props.min_keys {
            // Get predecessor
            let predecessor = self.get_predecessor(left_child);
            deref_node_mut(node_ptr).keys[key_idx] = predecessor.clone();
            self.remove_from_node(left_child, &predecessor);
            Some(key)
        } else if deref_node(right_child).keys.len() > self.props.min_keys {
            // Get successor
            let successor = self.get_successor(right_child);
            deref_node_mut(node_ptr).keys[key_idx] = successor.clone();
            self.remove_from_node(right_child, &successor);
            Some(key)
        } else {
            // Both children have minimum keys - merge
            self.merge_children(node_ptr, key_idx);
            self.remove_from_node(left_child, &key)
        }
    }

    fn ensure_child_has_enough_keys(&mut self, parent_ptr: NodePtr<T>, child_idx: usize) {
        let parent = deref_node(parent_ptr);

        // Try to borrow from left sibling
        if child_idx > 0 {
            let left_sibling = parent.children[child_idx - 1];
            if deref_node(left_sibling).keys.len() > self.props.min_keys {
                self.borrow_from_left_sibling(parent_ptr, child_idx);
                return;
            }
        }

        // Try to borrow from right sibling
        if child_idx < parent.children.len() - 1 {
            let right_sibling = parent.children[child_idx + 1];
            if deref_node(right_sibling).keys.len() > self.props.min_keys {
                self.borrow_from_right_sibling(parent_ptr, child_idx);
                return;
            }
        }

        // Can't borrow - must merge
        if child_idx < parent.children.len() - 1 {
            // Merge with right sibling
            self.merge_children(parent_ptr, child_idx);
        } else {
            // Merge with left sibling
            self.merge_children(parent_ptr, child_idx - 1);
        }
    }

    fn borrow_from_left_sibling(&mut self, parent_ptr: NodePtr<T>, child_idx: usize) {
        let parent = deref_node_mut(parent_ptr);
        let child_ptr = parent.children[child_idx];
        let left_sibling_ptr = parent.children[child_idx - 1];

        let separator_key = parent.keys[child_idx - 1].clone();

        // Move a key from left sibling through parent to child
        let left_sibling = deref_node_mut(left_sibling_ptr);
        let borrowed_key = left_sibling.keys.pop().unwrap();

        let borrowed_child = if !left_sibling.is_leaf() {
            Some(left_sibling.children.pop().unwrap())
        } else {
            None
        };

        parent.keys[child_idx - 1] = borrowed_key;

        let child = deref_node_mut(child_ptr);
        child.keys.insert(0, separator_key);

        if let Some(borrowed_child_ptr) = borrowed_child {
            child.children.insert(0, borrowed_child_ptr);
            deref_node_mut(borrowed_child_ptr).parent = Some(child_ptr);
        }
    }

    fn borrow_from_right_sibling(&mut self, parent_ptr: NodePtr<T>, child_idx: usize) {
        let parent = deref_node_mut(parent_ptr);
        let child_ptr = parent.children[child_idx];
        let right_sibling_ptr = parent.children[child_idx + 1];

        let separator_key = parent.keys[child_idx].clone();

        // Move a key from right sibling through parent to child
        let right_sibling = deref_node_mut(right_sibling_ptr);
        let borrowed_key = right_sibling.keys.remove(0).unwrap();

        let borrowed_child = if !right_sibling.is_leaf() {
            Some(right_sibling.children.remove(0).unwrap())
        } else {
            None
        };

        parent.keys[child_idx] = borrowed_key;

        let child = deref_node_mut(child_ptr);
        child.keys.push(separator_key);

        if let Some(borrowed_child_ptr) = borrowed_child {
            child.children.push(borrowed_child_ptr);
            deref_node_mut(borrowed_child_ptr).parent = Some(child_ptr);
        }
    }

    fn merge_children(&mut self, parent_ptr: NodePtr<T>, separator_idx: usize) {
        let parent = deref_node_mut(parent_ptr);
        let left_child_ptr = parent.children[separator_idx];
        let right_child_ptr = parent.children[separator_idx + 1];

        let separator_key = parent.keys.remove(separator_idx).unwrap();
        parent.children.remove(separator_idx + 1);

        // Merge right child into left child
        let right_child = deref_node_mut(right_child_ptr);
        let mut right_keys = mem::take(&mut right_child.keys);
        let mut right_children = mem::take(&mut right_child.children);

        let left_child = deref_node_mut(left_child_ptr);
        left_child.keys.push(separator_key);
        left_child.keys.extend(right_keys.drain_all());

        if !right_children.is_empty() {
            // Update parent pointers for the children we're moving
            for child_ptr in &right_children {
                deref_node_mut(*child_ptr).parent = Some(left_child_ptr);
            }
            left_child.children.extend(right_children.drain_all());
        }

        // Clean up the right child node
        Node::drop(right_child_ptr);
    }

    fn get_predecessor(&self, node_ptr: NodePtr<T>) -> T {
        let mut current = deref_node(node_ptr);
        while !current.is_leaf() {
            let last_child_idx = current.children.len() - 1;
            current = deref_node(current.children[last_child_idx]);
        }
        current.keys.last().unwrap().clone()
    }

    fn get_successor(&self, node_ptr: NodePtr<T>) -> T {
        let mut current = deref_node(node_ptr);
        while !current.is_leaf() {
            current = deref_node(current.children[0]);
        }
        current.keys[0].clone()
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

#[cfg(test)]
mod tests;

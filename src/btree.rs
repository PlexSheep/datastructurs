use std::mem;

use crate::vec::Vec;

pub const DEFAULT_DEGREE: usize = 100;

#[derive(Clone, Debug)]
pub struct BTree<T: Ord + Clone> {
    root: Node<T>,
    properties: BTreeProperties,
}

#[derive(Clone, Debug, Copy)]
pub struct BTreeProperties {
    degree: usize,
    max_keys: usize,
    mid_key_index: usize,
}

#[derive(Clone, Debug)]
struct Node<T> {
    keys: Vec<T>,
    children: Vec<Node<T>>,
}

impl BTreeProperties {
    #[must_use]
    fn new(degree: usize) -> Self {
        assert!(degree >= 3, "B-tree degree must be at least 3");
        Self {
            degree,
            max_keys: degree - 1,
            mid_key_index: (degree - 1) / 2,
        }
    }

    fn split_child<T: Ord + Clone>(&self, parent: &mut Node<T>, child_index: usize) {
        let child = &mut parent.children[child_index];

        let right_keys = child.keys.split_off(self.mid_key_index + 1);
        let middle_key = child.keys.pop().unwrap(); // We reinsert later

        let right_children = if !child.is_leaf() {
            Some(child.children.split_off(self.mid_key_index + 1))
        } else {
            None
        };

        let new_child_node = Node::new_with_data(self.degree, right_keys, right_children);

        parent.keys.insert(child_index, middle_key);
        parent.children.insert(child_index + 1, new_child_node);
    }

    #[must_use]
    fn is_full<T>(&self, node: &Node<T>) -> bool {
        node.keys.len() >= self.max_keys
    }

    #[must_use]
    fn find_insertion_index<T: Ord>(keys: &[T], key: &T) -> usize {
        match keys.binary_search(key) {
            Ok(idx) | Err(idx) => idx,
        }
    }

    fn insert_non_full<T: Ord + Clone>(&self, node: &mut Node<T>, key: T) {
        let index = Self::find_insertion_index(&node.keys, &key);

        if node.is_leaf() {
            node.keys.insert(index, key);
        } else if self.is_full(&node.children[index]) {
            self.split_child(node, index);
            // After split, determine which child to recurse into
            let final_index = if index < node.keys.len() && node.keys[index] < key {
                index + 1
            } else {
                index
            };
            self.insert_non_full(&mut node.children[final_index], key);
        } else {
            self.insert_non_full(&mut node.children[index], key);
        }
    }
}

impl<T> Node<T> {
    #[must_use]
    fn new(degree: usize) -> Self {
        Node {
            keys: Vec::with_capacity(degree - 1),
            children: Vec::with_capacity(degree),
        }
    }

    #[must_use]
    fn new_with_data(degree: usize, keys: Vec<T>, children: Option<Vec<Node<T>>>) -> Self {
        Node {
            keys,
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
            root: Node::new(degree),
            properties: BTreeProperties::new(degree),
        }
    }

    pub fn clear(&mut self) {
        self.root = Node::new(self.properties.degree);
    }

    pub fn insert(&mut self, key: T) {
        if self.properties.is_full(&self.root) {
            // Create new root and make old root its child
            let new_root = Node::new(self.properties.degree);
            let old_root = mem::replace(&mut self.root, new_root);
            self.root.children.push(old_root);
            self.properties.split_child(&mut self.root, 0);
        }
        self.properties.insert_non_full(&mut self.root, key);
    }

    #[must_use]
    pub fn contains(&self, key: &T) -> bool {
        let mut current = &self.root;
        loop {
            match current.keys.binary_search(key) {
                Ok(_) => return true,
                Err(idx) => {
                    if current.is_leaf() {
                        return false;
                    }
                    current = &current.children[idx];
                }
            }
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.root.keys.is_empty()
    }

    #[must_use]
    pub fn height(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let mut height = 1;
        let mut current = &self.root;
        while !current.is_leaf() {
            height += 1;
            current = &current.children[0];
        }
        height
    }

    // Iterator support - returns keys in sorted order
    pub fn iter(&self) -> BTreeIter<T> {
        BTreeIter::new(&self.root)
    }

    // Range query support
    pub fn range<'a>(&'a self, start: &T, end: &T) -> impl Iterator<Item = &'a T> {
        self.iter()
            .skip_while(move |&k| k < start)
            .take_while(move |&k| k <= end)
    }
}

// Simple iterator implementation
pub struct BTreeIter<'a, T> {
    stack: Vec<(&'a Node<T>, usize)>,
}

impl<'a, T> BTreeIter<'a, T> {
    fn new(root: &'a Node<T>) -> Self {
        let mut iter = BTreeIter { stack: Vec::new() };
        iter.push_left_path(root, 0);
        iter
    }

    fn push_left_path(&mut self, mut node: &'a Node<T>, start_idx: usize) {
        loop {
            self.stack.push((node, start_idx));
            if node.is_leaf() {
                break;
            }
            node = &node.children[start_idx];
        }
    }
}

impl<'a, T> Iterator for BTreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, idx)) = self.stack.pop() {
            if idx < node.keys.len() {
                let key = &node.keys[idx];

                // Prepare for next iteration
                if !node.is_leaf() && idx + 1 < node.children.len() {
                    self.push_left_path(&node.children[idx + 1], 0);
                }

                // Push current node back with incremented index
                if idx + 1 < node.keys.len() {
                    self.stack.push((node, idx + 1));
                }

                return Some(key);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create() {
        let _tree = BTree::<u32>::new(DEFAULT_DEGREE);
    }

    #[test]
    fn test_insert_and_contains() {
        let mut tree = BTree::<u32>::new(3); // Small degree for easier testing
        let data = &[10, 20, 5, 6, 12, 30, 7, 17];

        for &value in data {
            tree.insert(value);
        }

        for &value in data {
            assert!(tree.contains(&value), "Tree should contain {}", value);
        }

        assert!(!tree.contains(&999), "Tree should not contain 999");
    }

    #[test]
    fn test_iteration() {
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
    fn test_height() {
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
    fn test_moderate_dataset() {
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
    fn test_iter() {
        let data: Vec<_> = (0..9999).collect();
        let mut tree = BTree::new(DEFAULT_DEGREE);
        for d in &data {
            tree.insert(d);
        }

        for key in tree.iter() {
            assert!(data.contains(key))
        }
    }
}

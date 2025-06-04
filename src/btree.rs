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
    fn new(degree: usize) -> Self {
        Self {
            degree,
            max_keys: degree - 1,
            mid_key_index: (degree - 1) / 2,
        }
    }

    fn split_child<T: Ord + Clone>(&self, parent: &mut Node<T>, child_index: usize) {
        let child = &mut parent.children[child_index];
        let middle_key: T = child.keys[self.mid_key_index].clone();
        let right_keys = match child.keys.split_off(self.mid_key_index).split_first() {
            Some((_first, _others)) => {
                // We don't need _first, as it will move to parent node.
                _others.into()
            }
            None => Vec::<T>::with_capacity(self.max_keys),
        };
        let right_children = if !child.is_leaf() {
            Some(child.children.split_off(self.mid_key_index + 1))
        } else {
            None
        };
        let new_child_node: Node<T> = Node::new(self.degree, Some(right_keys), right_children);

        parent.keys.insert(child_index, middle_key);
        parent.children.insert(child_index + 1, new_child_node);
    }

    fn is_maxed_out<T: Ord>(&self, node: &Node<T>) -> bool {
        node.keys.len() == self.max_keys
    }

    fn insert_non_full<T: Ord + Clone>(&mut self, node: &mut Node<T>, key: T) {
        let mut index: isize = isize::try_from(node.keys.len()).ok().unwrap() - 1;
        while index >= 0 && node.keys[index as usize] >= key {
            index -= 1;
        }

        let mut u_index: usize = usize::try_from(index + 1).ok().unwrap();
        if node.is_leaf() {
            // Just insert it, as we know this method will be called only when node is not full
            node.keys.insert(u_index, key);
        } else {
            if self.is_maxed_out(&node.children[u_index]) {
                self.split_child(node, u_index);
                if node.keys[u_index] < key {
                    u_index += 1;
                }
            }

            self.insert_non_full(&mut node.children[u_index], key);
        }
    }
}

impl<T> Node<T>
where
    T: Ord,
{
    fn new(degree: usize, keys: Option<Vec<T>>, children: Option<Vec<Node<T>>>) -> Self {
        Node {
            keys: match keys {
                Some(keys) => keys,
                None => Vec::with_capacity(degree - 1),
            },
            children: match children {
                Some(children) => children,
                None => Vec::with_capacity(degree),
            },
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

impl<T: Ord + Clone> BTree<T> {
    pub fn new(branch_factor: usize) -> Self {
        let degree = 2 * branch_factor;
        Self {
            root: Node::new(degree, None, None),
            properties: BTreeProperties::new(degree),
        }
    }

    pub fn clear(&mut self) {
        self.root = Node::new(self.properties.degree, None, None);
    }

    pub fn insert(&mut self, key: T) {
        if self.properties.is_maxed_out(&self.root) {
            // Create an empty root and split the old root...
            let mut new_root = Node::new(self.properties.degree, None, None);
            mem::swap(&mut new_root, &mut self.root);
            self.root.children.insert(0, new_root);
            self.properties.split_child(&mut self.root, 0);
        }
        self.properties.insert_non_full(&mut self.root, key)
    }

    #[must_use]
    pub fn has(&self, key: T) -> bool {
        let mut current_node = &self.root;
        loop {
            match current_node.keys.binary_search(&key) {
                Ok(_) => return true,
                Err(idx) => {
                    if current_node.is_leaf() {
                        return false;
                    }
                    current_node = &current_node.children[idx];
                }
            }
        }
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
    fn test_insert_easy() {
        let mut tree = BTree::<u32>::new(DEFAULT_DEGREE);
        let data = &[19, 125, 25, 16, 2, 73, 384, 435, 12924, 42, 125251, 2548];

        for d in data {
            tree.insert(*d)
        }

        for d in data {
            assert!(tree.has(*d))
        }
    }

    #[test]
    fn test_insert_many() {
        let mut tree = BTree::<u32>::new(DEFAULT_DEGREE);
        let mut data = vec![19, 125, 25, 16, 2, 73, 384, 435, 12924, 42, 125251, 2548];

        for _ in 0..10 {
            data.extend(data.clone());
        }

        // data has 12288 elements here! This is a lot, but should be reasonably possible for a btree.
        println!("len of data: {}", data.len());

        for d in &data {
            tree.insert(*d)
        }

        for d in &data {
            assert!(tree.has(*d))
        }
    }

    #[test]
    fn test_insert_large() {
        let mut tree = BTree::<u32>::new(DEFAULT_DEGREE);
        let base_data = vec![19, 125, 25, 16, 2, 73, 384, 435, 12924, 42, 125251, 2548];
        let mut data = base_data.clone();

        // Create an even larger dataset to really stress test the iterative approach
        for _ in 0..15 {
            data.extend(data.clone());
        }

        println!("len of data: {}", data.len());

        for d in &data {
            tree.insert(*d)
        }

        // Verify a sample of the data
        for d in &base_data {
            assert!(tree.has(*d))
        }
    }

    #[test]
    fn test_insert_very_large() {
        let mut tree = BTree::<u32>::new(DEFAULT_DEGREE);
        let base_data = vec![19, 125, 25, 16, 2, 73, 384, 435, 12924, 42, 125251, 2548];
        let mut data = base_data.clone();

        // Create an even larger dataset to really stress test the iterative approach
        for _ in 0..27 {
            data.extend(data.clone());
        }

        // 5 GiB of this vector
        println!("len of data: {}", data.len());

        for d in &data {
            tree.insert(*d)
        }

        // Verify a sample of the data
        for d in &base_data {
            assert!(tree.has(*d))
        }
    }
}

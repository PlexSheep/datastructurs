use std::mem;

use crate::btree::{BTreeSet, Node, NodePtr, deref_node, deref_node_mut};

mod impls;

#[derive(Debug, Clone)]
struct MapPair<K, V> {
    key: K,
    value: V,
}

// NOTE: the key and value must be Clone because of BTreeSet implementation details. BTreeSet should
// eventually be remfactored to remove the Clone dependency
#[derive(Clone)]
pub struct BTreeMap<K: Ord + Clone, V: Clone> {
    set: BTreeSet<MapPair<K, V>>,
}

impl<K: Ord + Clone, V: Clone> BTreeMap<K, V> {
    #[must_use]
    pub fn new(branch_factor: usize) -> Self {
        let set = BTreeSet::new(branch_factor);
        Self { set }
    }

    pub fn clear(&mut self) {
        *self = Self::new(self.set.props.degree * 2)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let pair = MapPair { key, value };
        let r = if self.set.contains(&pair) {
            self.set.remove(&pair)
        } else {
            None
        };
        self.set.insert(pair);
        r.map(|r| r.value)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.set.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> {
        let this = &self.set;
        let mut current = deref_node(this.root);
        loop {
            match current.keys.binary_search_by(|k| k.key.cmp(key)) {
                Ok(idx) => return Some(&current.keys[idx].value),
                Err(idx) => {
                    if current.is_leaf() {
                        return None;
                    }
                    current = deref_node(current.children[idx]);
                    continue;
                }
            }
        }
    }

    #[must_use]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let this = &mut self.set;
        let mut current = deref_node_mut(this.root);
        loop {
            match current.keys.binary_search_by(|k| k.key.cmp(key)) {
                Ok(idx) => return Some(&mut current.keys[idx].value),
                Err(idx) => {
                    if current.is_leaf() {
                        return None;
                    }
                    current = deref_node_mut(current.children[idx]);
                    continue;
                }
            }
        }
    }

    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        let mut current = deref_node(self.set.root);
        loop {
            match current.keys.binary_search_by(|k| k.key.cmp(key)) {
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
}

// removing keys
// this is so fucking complicated
impl<K: Ord + Clone, V: Clone> BTreeMap<K, V> {
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let result = self.remove_from_node(self.set.root, key);

        // Handle root underflow - if root is empty but has children, promote the only child
        let root_node = deref_node(self.set.root);
        if root_node.keys.is_empty() && !root_node.children.is_empty() {
            let old_root = self.set.root;
            self.set.root = root_node.children[0];

            // Update the new root's parent to None
            deref_node_mut(self.set.root).parent = None;

            // Prevent the old root from dropping its children
            deref_node_mut(old_root).children.clear();
            Node::drop(old_root);
        }

        if result.is_some() {
            self.set.props.len -= 1;
        }
        result.map(|p| p.value)
    }

    fn remove_from_node(
        &mut self,
        node_ptr: NodePtr<MapPair<K, V>>,
        key: &K,
    ) -> Option<MapPair<K, V>> {
        let node = deref_node_mut(node_ptr);

        match node.keys.binary_search_by(|k| k.key.cmp(key)) {
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
                    if deref_node(child_ptr).keys.len() <= self.set.props.min_keys {
                        self.ensure_child_has_enough_keys(node_ptr, idx);

                        // After rebalancing, we need to search again as indices may have changed
                        let node = deref_node(node_ptr);
                        let new_idx = match node.keys.binary_search_by(|k| k.key.cmp(key)) {
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

    fn remove_from_internal_node(
        &mut self,
        node_ptr: NodePtr<MapPair<K, V>>,
        key_idx: usize,
    ) -> Option<MapPair<K, V>> {
        let node = deref_node(node_ptr);
        let key = node.keys[key_idx].clone();

        let left_child = node.children[key_idx];
        let right_child = node.children[key_idx + 1];

        if deref_node(left_child).keys.len() > self.set.props.min_keys {
            // Get predecessor
            let predecessor = self.get_predecessor(left_child);
            deref_node_mut(node_ptr).keys[key_idx] = predecessor.clone();
            self.remove_from_node(left_child, &predecessor.key);
            Some(key)
        } else if deref_node(right_child).keys.len() > self.set.props.min_keys {
            // Get successor
            let successor = self.get_successor(right_child);
            deref_node_mut(node_ptr).keys[key_idx] = successor.clone();
            self.remove_from_node(right_child, &successor.key);
            Some(key)
        } else {
            // Both children have minimum keys - merge
            self.merge_children(node_ptr, key_idx);
            self.remove_from_node(left_child, &key.key)
        }
    }

    fn ensure_child_has_enough_keys(
        &mut self,
        parent_ptr: NodePtr<MapPair<K, V>>,
        child_idx: usize,
    ) {
        let parent = deref_node(parent_ptr);

        // Try to borrow from left sibling
        if child_idx > 0 {
            let left_sibling = parent.children[child_idx - 1];
            if deref_node(left_sibling).keys.len() > self.set.props.min_keys {
                self.borrow_from_left_sibling(parent_ptr, child_idx);
                return;
            }
        }

        // Try to borrow from right sibling
        if child_idx < parent.children.len() - 1 {
            let right_sibling = parent.children[child_idx + 1];
            if deref_node(right_sibling).keys.len() > self.set.props.min_keys {
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

    fn borrow_from_left_sibling(&mut self, parent_ptr: NodePtr<MapPair<K, V>>, child_idx: usize) {
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

    fn borrow_from_right_sibling(&mut self, parent_ptr: NodePtr<MapPair<K, V>>, child_idx: usize) {
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

    fn merge_children(&mut self, parent_ptr: NodePtr<MapPair<K, V>>, separator_idx: usize) {
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

    fn get_predecessor(&self, node_ptr: NodePtr<MapPair<K, V>>) -> MapPair<K, V> {
        let mut current = deref_node(node_ptr);
        while !current.is_leaf() {
            let last_child_idx = current.children.len() - 1;
            current = deref_node(current.children[last_child_idx]);
        }
        current.keys.last().unwrap().clone()
    }

    fn get_successor(&self, node_ptr: NodePtr<MapPair<K, V>>) -> MapPair<K, V> {
        let mut current = deref_node(node_ptr);
        while !current.is_leaf() {
            current = deref_node(current.children[0]);
        }
        current.keys[0].clone()
    }
}

#[cfg(test)]
mod tests;

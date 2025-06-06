use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use super::{BTree, Node, NodePtr, deref_node, deref_node_mut};

impl<T: Ord> Drop for Node<T> {
    fn drop(&mut self) {
        for child_ptr in &self.children {
            Node::drop(*child_ptr);
        }
    }
}

impl<T: Ord + Clone> Drop for BTree<T> {
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
    pub(crate) fn new(root_ptr: &'a NodePtr<T>) -> Self {
        let mut iter = BTreeIter {
            stack: Vec::new(),
            marker: PhantomData,
        };
        iter.push_left_path(root_ptr, 0);
        iter
    }

    fn push_left_path(&mut self, node_ptr: &'a NodePtr<T>, start_idx: usize) {
        let mut node = deref_node_mut(*node_ptr);
        loop {
            self.stack.push((node.as_ptr(), start_idx));
            if node.is_leaf() {
                break;
            }
            node = deref_node_mut(node.children[start_idx]);
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

                if idx + 1 < node.keys.len() {
                    self.stack.push((node_ptr, idx + 1));
                }

                if !node.is_leaf() && idx + 1 < node.children.len() {
                    self.push_left_path(&node.children[idx + 1], 0);
                }

                return Some(key);
            }
        }
        None
    }
}

impl<T: Ord + Clone + Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut children = Vec::with_capacity(self.children.len());
        for child_ptr in &self.children {
            children.push(deref_node(*child_ptr));
        }

        f.debug_struct("Node")
            .field("keys", &self.keys)
            .field("children", &children)
            .field("parent", &self.parent)
            .field("_addr", &self.as_ptr())
            .finish()
    }
}

impl<T: Ord + Clone + Debug> Debug for BTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BTree")
            .field("props", &self.props)
            .field("Nodes", deref_node(self.root))
            .finish()
    }
}

impl<T: Ord + Clone + Display + Debug> Display for BTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn render_child<T: Ord + Clone + Display + Debug>(
            node: &Node<T>,
            depth: usize,
            buffer: &mut String,
        ) {
            if depth > 0 {
                buffer.push_str(&format!("{:>1$}", "|-", (depth) * 2));
            }
            buffer.push_str(&format!("{:?}\n", node.keys));
            for child_ptr in &node.children {
                let node = deref_node(*child_ptr);
                render_child(node, depth + 1, buffer);
            }
        }

        let mut buf = String::new();
        let root = deref_node(self.root);

        render_child(root, 0, &mut buf);

        write!(f, "{buf}")
    }
}

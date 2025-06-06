use super::*;

#[test]
fn test_btree_create() {
    let _tree = BTree::<u32>::new(DEFAULT_BRANCH_FACTOR);
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
    let mut tree = BTree::new(DEFAULT_BRANCH_FACTOR);
    for d in &data {
        tree.insert(d);
    }

    for key in tree.iter() {
        assert!(data.contains(key))
    }
}

#[test]
#[ignore = "too work heavy"]
fn test_btree_stress() {
    let mut tree = BTree::new(DEFAULT_BRANCH_FACTOR);
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

#[test]
fn test_btree_simple_remove() {
    let mut tree = BTree::new(2); // degree=4, min_keys=2, max_keys=3
    assert_eq!(tree.node_count(), 1);
    tree.insert(1337);
    assert_eq!(tree.node_count(), 1);

    for i in 1..=7 {
        tree.insert(i);
    }
    assert_eq!(tree.len(), 8);
    assert_eq!(tree.node_count(), 5);
    assert_eq!(tree.depth(), 2);

    tree.remove(&1); // node underflow

    assert!(!tree.contains(&1));
    for i in 2..=7 {
        assert!(tree.contains(&i))
    }
    assert_eq!(tree.len(), 7);
    assert_eq!(tree.depth(), 2);
    assert_eq!(tree.node_count(), 4);

    tree.insert(19);
    assert_eq!(tree.len(), 8);
    assert_eq!(tree.depth(), 2);
    assert_eq!(tree.node_count(), 4);
    assert!(tree.contains(&19));

    tree.insert(19); // now it's there two times!
    assert_eq!(tree.len(), 9);
    assert_eq!(tree.depth(), 2);
    assert_eq!(tree.node_count(), 5);
    assert!(tree.contains(&19));
}

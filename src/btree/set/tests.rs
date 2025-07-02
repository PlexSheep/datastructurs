use crate::{btree::DEFAULT_BRANCH_FACTOR, trace};

use super::*;

#[test]
fn test_btree_set_create() {
    let _tree = BTreeSet::<u32>::new(DEFAULT_BRANCH_FACTOR);
}

#[test]
fn test_btree_set_insert_contains_remove_in_order() {
    let mut tree = BTreeSet::<u32>::new(3); // Small degree for easier testing
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
fn test_btree_set_insert_contains_remove_out_of_order() {
    let mut tree = BTreeSet::<u32>::new(3); // Small degree for easier testing
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
fn test_btree_set_iteration() {
    let mut tree = BTreeSet::new(3);
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
fn test_btree_set_height() {
    let mut tree = BTreeSet::new(3);
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
fn test_btree_set_moderate_dataset() {
    let mut tree = BTreeSet::<u32>::new(50);
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
fn test_btree_set_iter() {
    let data: Vec<_> = (0..9999).collect();
    let mut tree = BTreeSet::new(DEFAULT_BRANCH_FACTOR);
    for d in &data {
        tree.insert(d);
    }

    for key in tree.iter() {
        assert!(data.contains(key))
    }
}

#[test]
#[ignore = "too work heavy"]
fn test_btree_set_stress() {
    let mut tree = BTreeSet::new(DEFAULT_BRANCH_FACTOR);
    let range = 0..5_000_000;
    for d in range.clone() {
        tree.insert(d);
    }
    trace!("Tree height: {}", tree.height());
    trace!("Tree len: {}", tree.len());

    for key in range.clone() {
        assert!(tree.contains(&key))
    }

    for key in range.into_iter().rev() {
        assert_eq!(tree.pop_last().unwrap(), key)
    }
    assert!(tree.is_empty())
}

#[test]
fn test_btree_set_simple_remove() {
    let mut tree = BTreeSet::new(2); // degree=4, min_keys=2, max_keys=3
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

#[test]
fn test_btree_set_empty_operations() {
    let tree = BTreeSet::<i32>::new(3);
    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
    assert_eq!(tree.height(), 0);
    assert_eq!(tree.first(), None);
    assert_eq!(tree.last(), None);
    assert!(!tree.contains(&42));
}

#[test]
fn test_btree_set_single_element() {
    let mut tree = BTreeSet::new(3);
    tree.insert(42);

    assert!(!tree.is_empty());
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.height(), 1);
    assert_eq!(tree.first(), Some(&42));
    assert_eq!(tree.last(), Some(&42));
    assert!(tree.contains(&42));
    assert!(!tree.contains(&99));
}

#[test]
fn test_btree_set_ordered_insertion() {
    let mut tree = BTreeSet::new(3);
    let data = (1..=100).collect::<std::vec::Vec<_>>();

    for &x in &data {
        tree.insert(x);
    }

    assert_eq!(tree.len(), data.len());
    for &x in &data {
        assert!(tree.contains(&x));
    }

    // Verify iteration returns sorted order
    let collected: std::vec::Vec<_> = tree.iter().cloned().collect();
    assert_eq!(collected, data);
}

#[test]
fn test_btree_set_reverse_insertion() {
    let mut tree = BTreeSet::new(3);
    let data = (1..=100).rev().collect::<std::vec::Vec<_>>();

    for &x in &data {
        tree.insert(x);
    }

    // Should still iterate in sorted order
    let collected: std::vec::Vec<_> = tree.iter().cloned().collect();
    let expected = (1..=100).collect::<std::vec::Vec<_>>();
    assert_eq!(collected, expected);
}

#[test]
fn test_btree_set_random_insertion() {
    let mut tree = BTreeSet::new(3);
    let mut data = (1..=50).collect::<std::vec::Vec<_>>();

    // Shuffle the data (simple pseudo-random shuffle)
    for i in 0..data.len() {
        let j = (i * 17 + 3) % data.len();
        data.swap(i, j);
    }

    for &x in &data {
        tree.insert(x);
    }

    // Verify all elements present and sorted
    let collected: std::vec::Vec<_> = tree.iter().cloned().collect();
    let expected = (1..=50).collect::<std::vec::Vec<_>>();
    assert_eq!(collected, expected);
}

#[test]
fn test_btree_set_duplicates() {
    let mut tree = BTreeSet::new(3);

    tree.insert(5);
    tree.insert(5);
    tree.insert(5);

    assert_eq!(tree.len(), 3);
    assert!(tree.contains(&5));

    // Should find all three instances in iteration
    let count = tree.iter().filter(|&&x| x == 5).count();
    assert_eq!(count, 3);
}

#[test]
fn test_btree_set_first_last() {
    let mut tree = BTreeSet::new(3);
    let data = vec![50, 25, 75, 10, 30, 60, 80];

    for &x in &data {
        tree.insert(x);
    }

    assert_eq!(tree.first(), Some(&10));
    assert_eq!(tree.last(), Some(&80));
}

#[test]
fn test_btree_set_pop_operations() {
    let mut tree = BTreeSet::new(3);
    let data = vec![1, 3, 5, 7, 9];

    for &x in &data {
        tree.insert(x);
    }

    assert_eq!(tree.pop_first(), Some(1));
    assert_eq!(tree.pop_last(), Some(9));
    assert_eq!(tree.len(), 3);

    assert_eq!(tree.first(), Some(&3));
    assert_eq!(tree.last(), Some(&7));
}

#[test]
fn test_btree_set_remove_all_elements() {
    let mut tree = BTreeSet::new(3);
    let data = (1..=20).collect::<std::vec::Vec<_>>();

    for &x in &data {
        tree.insert(x);
    }

    // Remove all elements in random order
    let mut to_remove = data.clone();
    for i in 0..to_remove.len() {
        let j = (i * 7 + 11) % to_remove.len();
        to_remove.swap(i, j);
    }

    for &x in &to_remove {
        assert!(tree.contains(&x));
        assert_eq!(tree.remove(&x), Some(x));
        assert!(!tree.contains(&x));
    }

    assert!(tree.is_empty());
}

#[test]
fn test_btree_set_height_characteristics() {
    let mut tree = BTreeSet::new(50); // Large branch factor

    // Insert many elements and verify height grows logarithmically
    for i in 1..=1000 {
        tree.insert(i);
    }

    let height = tree.height();
    assert!(height > 0);
    assert!(height < 10); // Should be quite shallow with branch factor 100

    trace!("Tree with 1000 elements has height: {}", height);
    trace!("Tree has {} nodes", tree.node_count());
}

#[test]
fn test_btree_set_range_iteration() {
    let mut tree = BTreeSet::new(3);
    let data = (1..=100).collect::<std::vec::Vec<_>>();

    for &x in &data {
        tree.insert(x);
    }

    // Test range iteration
    let range_25_75: std::vec::Vec<_> = tree.range(&25, &75).cloned().collect();
    let expected = (25..=75).collect::<std::vec::Vec<_>>();
    assert_eq!(range_25_75, expected);

    // Test edge cases
    let range_1_5: std::vec::Vec<_> = tree.range(&1, &5).cloned().collect();
    assert_eq!(range_1_5, vec![1, 2, 3, 4, 5]);

    let range_95_100: std::vec::Vec<_> = tree.range(&95, &100).cloned().collect();
    assert_eq!(range_95_100, vec![95, 96, 97, 98, 99, 100]);
}

#[test]
fn test_btree_set_clear() {
    let mut tree = BTreeSet::new(3);

    for i in 1..=100 {
        tree.insert(i);
    }
    assert!(!tree.is_empty());

    tree.clear();
    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
    assert_eq!(tree.height(), 0);

    // Should be usable after clear
    tree.insert(42);
    assert_eq!(tree.len(), 1);
    assert!(tree.contains(&42));
}

#[test]
fn test_btree_set_different_branch_factors() {
    for branch_factor in [2, 3, 5, 10, 50, 100, 200, 1000].iter() {
        let mut tree = BTreeSet::new(*branch_factor);
        let data = (1..=100).collect::<std::vec::Vec<_>>();

        for &x in &data {
            tree.insert(x);
        }

        assert_eq!(tree.len(), 100);

        // Verify all elements are present and sorted
        let collected: std::vec::Vec<_> = tree.iter().cloned().collect();
        assert_eq!(collected, data);

        trace!(
            "Branch factor {}: height = {}, nodes = {}",
            branch_factor,
            tree.height(),
            tree.node_count()
        );
    }
}

#[test]
fn test_btree_set_string_operations() {
    let mut tree = BTreeSet::new(3);
    let words = vec!["apple", "banana", "cherry", "date", "elderberry"];

    for &word in &words {
        tree.insert(word.to_string());
    }

    // Should be sorted alphabetically
    let collected: std::vec::Vec<_> = tree.iter().collect();
    assert_eq!(
        collected,
        vec![
            &"apple".to_string(),
            &"banana".to_string(),
            &"cherry".to_string(),
            &"date".to_string(),
            &"elderberry".to_string()
        ]
    );
}

#[test]
fn test_btree_set_memory_intensive() {
    let mut tree = BTreeSet::new(DEFAULT_BRANCH_FACTOR);
    let size = 100_000;

    // Insert large dataset
    for i in 0..size {
        tree.insert(i);
    }

    assert_eq!(tree.len(), size);

    // Test random access
    for i in (0..size).step_by(1000) {
        assert!(tree.contains(&i));
    }

    // Test iteration (just first 100 to avoid slow test)
    let first_100: std::vec::Vec<_> = tree.iter().take(100).cloned().collect();
    let expected = (0..100).collect::<std::vec::Vec<_>>();
    assert_eq!(first_100, expected);

    trace!(
        "Large tree stats: height = {}, nodes = {}",
        tree.height(),
        tree.node_count()
    );
}

#[test]
fn test_btree_set_edge_removals() {
    let mut tree = BTreeSet::new(2); // Small degree to force more splits/merges

    // Insert sequence that will cause complex tree structure
    for i in [10, 5, 15, 2, 7, 12, 18, 1, 3, 6, 8, 11, 13, 16, 20] {
        tree.insert(i);
    }

    let initial_len = tree.len();

    // Remove elements that might cause underflow and rebalancing
    assert_eq!(tree.remove(&1), Some(1));
    assert_eq!(tree.remove(&20), Some(20));
    assert_eq!(tree.remove(&10), Some(10)); // Remove from internal node

    assert_eq!(tree.len(), initial_len - 3);
    assert!(!tree.contains(&1));
    assert!(!tree.contains(&20));
    assert!(!tree.contains(&10));

    // Verify remaining elements are still accessible and sorted
    let remaining: std::vec::Vec<_> = tree.iter().cloned().collect();
    let expected = vec![2, 3, 5, 6, 7, 8, 11, 12, 13, 15, 16, 18];
    assert_eq!(remaining, expected);
}

use datastructurs::btree::{BTreeSet, DEFAULT_BRANCH_FACTOR};

fn main() {
    let mut tree = BTreeSet::new(DEFAULT_BRANCH_FACTOR);
    for i in 0..50_000 {
        tree.insert(i);
        assert!(tree.contains(&i));
    }

    for i in &[12, 43_312, 24_032, 12_000, 12_001, 12_002] {
        tree.remove(i);
        assert!(!tree.contains(i));
    }
    println!("{tree}");

    for i in 0..50_000 {
        tree.remove(&i);
    }

    assert!(tree.is_empty());

    tree.clear();
    assert!(tree.is_empty())
}

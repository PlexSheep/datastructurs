use crate::btree::{BTreeMap, DEFAULT_BRANCH_FACTOR};

#[test]
fn test_btree_map_new() {
    let _bm: BTreeMap<u32, String> = BTreeMap::new(DEFAULT_BRANCH_FACTOR);
}

#[test]
fn test_btree_map_insert_contains_remove() {
    let data = &[10, 20, 5, 6, 12, 30, 7, 17];
    let mut bm: BTreeMap<u32, String> = BTreeMap::new(3);

    fn f(i: u32) -> String {
        format!("Number is {i}")
    }

    for i in data {
        assert_eq!(None, bm.insert(*i, f(*i)))
    }

    for i in data {
        assert!(bm.contains_key(i))
    }

    for i in data {
        let should_be = f(*i);
        assert_eq!(Some(should_be), bm.remove(i))
    }
}

#[test]
fn test_btree_map_insert_weird_key() {
    let data = &[10, 20, 5, 6, 12, 30, 7, 17];
    let mut bm: BTreeMap<String, u32> = BTreeMap::new(3);

    fn f(i: u32) -> String {
        format!("Number is {i}")
    }

    for i in data {
        assert_eq!(None, bm.insert(f(*i), *i))
    }

    for i in data {
        assert!(bm.contains_key(&f(*i)))
    }

    for i in data {
        assert_eq!(Some(*i), bm.remove(&f(*i)))
    }
}

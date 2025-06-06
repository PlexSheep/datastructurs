use super::*;

#[test]
fn test_vec_create() {
    let _v = Vec::<u64>::new();
}

#[test]
fn test_vec_pushpop_num() {
    let mut v = Vec::new();
    let vals = &[19, 9, 14, 255, 19191919, 13890, 21521, 1251, 6216, 1830];

    for val in vals {
        v.push(*val);
    }
    for val in vals.iter().rev() {
        assert_eq!(v.pop().unwrap(), *val);
    }
}

#[test]
fn test_vec_pushpop_str() {
    let mut v = Vec::new();
    let vals = &["AAAA", "ABBAB", "BBABBABBAJJJ"];

    for val in vals {
        v.push(*val);
    }
    for val in vals.iter().rev() {
        assert_eq!(v.pop().unwrap(), *val);
    }
}

#[test]
fn test_vec_pushindex_num() {
    let mut v = Vec::new();
    let vals = &[19, 9, 14, 255, 19191919, 13890, 21521, 1251, 6216, 1830];

    for val in vals {
        v.push(*val);
    }
    for (idx, val) in vals.iter().enumerate() {
        assert_eq!(v[idx], *val);
    }
}

#[test]
fn test_vec_pushindex_str() {
    let mut v = Vec::new();
    let vals = &["AAAA", "ABBAB", "BBABBABBAJJJ"];

    for val in vals {
        v.push(*val);
    }
    for (idx, val) in vals.iter().enumerate() {
        assert_eq!(v[idx], *val);
    }
}

#[test]
fn test_vec_capacity_growth() {
    let mut v = Vec::new();
    assert_eq!(v.capacity(), 0);

    v.push(1);
    assert!(v.capacity() >= 1);
    let first_cap = v.capacity();

    // Fill to capacity
    while v.len() < v.capacity() {
        v.push(v.len() as i32);
    }

    // Next push should trigger growth
    v.push(999);
    assert!(v.capacity() > first_cap);
    assert_eq!(v[v.len() - 1], 999);
}

#[test]
fn test_vec_with_capacity() {
    let v = Vec::<i32>::with_capacity(100);
    assert!(v.capacity() >= 100);
    assert_eq!(v.len(), 0);
    assert!(v.is_empty());
}

#[test]
fn test_vec_insert_at_various_positions() {
    let mut v = Vec::new();

    // Insert into empty vec
    v.insert(0, 10);
    assert_eq!(v[0], 10);
    assert_eq!(v.len(), 1);

    // Insert at beginning
    v.insert(0, 5);
    assert_eq!(v[0], 5);
    assert_eq!(v[1], 10);

    // Insert at end (equivalent to push)
    v.insert(v.len(), 15);
    assert_eq!(v[2], 15);

    // Insert in middle
    v.insert(1, 7);
    assert_eq!(v, Vec::from(&[5, 7, 10, 15][..]));
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_vec_insert_out_of_bounds() {
    let mut v = Vec::new();
    v.push(1);
    v.insert(5, 999); // Should panic
}

#[test]
fn test_vec_remove_various_positions() {
    let mut v = Vec::from(&[1, 2, 3, 4, 5][..]);

    // Remove from middle
    assert_eq!(v.remove(2), Some(3));
    assert_eq!(v, Vec::from(&[1, 2, 4, 5][..]));

    // Remove from beginning
    assert_eq!(v.remove(0), Some(1));
    assert_eq!(v, Vec::from(&[2, 4, 5][..]));

    // Remove from end
    assert_eq!(v.remove(v.len() - 1), Some(5));
    assert_eq!(v, Vec::from(&[2, 4][..]));

    // Remove out of bounds
    assert_eq!(v.remove(10), None);
}

#[test]
fn test_vec_clear() {
    let d = &[1, 2, 3, 4, 5][..];
    let mut v = Vec::from(d);
    assert!(!v.is_empty());
    for (i, d) in d.iter().enumerate() {
        assert_eq!(v[i], *d)
    }

    v.clear();
    assert!(v.is_empty());
    assert_eq!(v.len(), 0);

    // Should be able to use after clear
    v.push(42);
    assert_eq!(v[0], 42);
    assert_eq!(v.len(), 1);
}

#[test]
fn test_vec_split_off() {
    let mut v = Vec::from(&[1, 2, 3, 4, 5, 6][..]);
    let other = v.split_off(3);

    assert_eq!(v, Vec::from(&[1, 2, 3][..]));
    assert_eq!(other, Vec::from(&[4, 5, 6][..]));
}

#[test]
fn test_vec_drain_all() {
    let mut v = Vec::from(&[1, 2, 3, 4, 5][..]);
    let drained: Vec<i32> = v.drain_all().collect();

    assert_eq!(drained, Vec::from(&[1, 2, 3, 4, 5][..]));
    assert!(v.is_empty());
}

#[test]
fn test_vec_iterators() {
    let v = Vec::from(&[1, 2, 3, 4, 5][..]);

    // Test IntoIterator for &Vec
    let mut sum = 0;
    for &x in &v {
        sum += x;
    }
    assert_eq!(sum, 15);

    // Test IntoIterator for Vec (consuming)
    let sum: i32 = v.into_iter().sum();
    assert_eq!(sum, 15);
}

#[test]
fn test_vec_from_iter() {
    let original = vec![1, 2, 3, 4, 5];
    let custom_vec: Vec<i32> = original.into_iter().collect();
    assert_eq!(custom_vec, Vec::from(&[1, 2, 3, 4, 5][..]));
}

#[test]
fn test_vec_extend() {
    let mut v = Vec::from(&[1, 2][..]);
    v.extend(vec![3, 4, 5]);
    assert_eq!(v, Vec::from(&[1, 2, 3, 4, 5][..]));
}

#[test]
fn test_vec_large_data() {
    let mut v = Vec::new();
    let size = 10_000;

    // Test push performance and correctness
    for i in 0..size {
        v.push(i);
    }

    assert_eq!(v.len(), size);
    for i in 0..size {
        assert_eq!(v[i], i);
    }

    // Test pop performance
    for i in (0..size).rev() {
        assert_eq!(v.pop(), Some(i));
    }
    assert!(v.is_empty());
}

#[test]
fn test_vec_string_operations() {
    let mut v = Vec::new();
    let strings = vec!["hello", "world", "rust", "is", "awesome"];

    for s in &strings {
        v.push(s.to_string());
    }

    assert_eq!(v.len(), strings.len());
    for (i, s) in strings.iter().enumerate() {
        assert_eq!(v[i], s.to_string());
    }
}

#[test]
fn test_vec_debug_repr() {
    let v = Vec::from(&[19, 1, 24, 13, 25, 25][..]);
    assert_eq!(format!("{v:?}"), "[19, 1, 24, 13, 25, 25]")
}

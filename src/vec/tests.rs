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

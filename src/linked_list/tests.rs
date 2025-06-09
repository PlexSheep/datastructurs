use super::*;

#[test]
fn test_ll_push_front() {
    let mut ll = LinkedList::new();
    for i in 0..12 {
        ll.push_front(i);
    }
    for i in 0..12 {
        println!("i={i}\n{}", ll.debug_nodes());
        assert!(ll.contains(&i))
    }
}

#[test]
fn test_ll_push_back() {
    let mut ll = LinkedList::new();
    for i in 0..12 {
        ll.push_back(i);
    }
    for i in 0..12 {
        println!("i={i}\n{}", ll.debug_nodes());
        assert!(ll.contains(&i))
    }
}

#[test]
fn test_ll_push_pop_front() {
    let mut ll = LinkedList::new();
    for i in 0..12 {
        ll.push_front(i);
    }
    for i in 0..12 {
        assert!(ll.contains(&i))
    }
    for i in 0..12 {
        println!("i={i}\n{}", ll.debug_nodes());
        ll.pop_front();
    }
    assert!(ll.is_empty())
}

#[test]
fn test_ll_push_pop_back() {
    let mut ll = LinkedList::new();
    for i in 0..12 {
        ll.push_back(i);
    }
    for i in 0..12 {
        assert!(ll.contains(&i))
    }
    for i in 0..12 {
        println!("i={i}\n{}", ll.debug_nodes());
        ll.pop_back();
    }
    assert!(ll.is_empty())
}

#[test]
fn test_ll_ins_only_one_thing() {
    let mut ll = LinkedList::new();
    ll.push_front(1);
    assert!(ll.contains(&1));
    assert_eq!(ll.len(), 1);
    assert!(!ll.is_empty());
}

#[test]
fn test_ll_ins_multiple() {
    let mut ll = LinkedList::new();

    ll.push_front(1);
    ll.push_front(2);
    println!("{ll:?}");
    println!("{}", ll.debug_nodes());
    ll.push_front(3);
    println!("{ll:?}");
    println!("{}", ll.debug_nodes());
    ll.push_front(4);
    println!("{ll:?}");
    println!("{}", ll.debug_nodes());

    assert!(ll.contains(&4));
    assert!(ll.contains(&3));
    assert!(ll.contains(&2));
    assert!(ll.contains(&1));
}

#[test]
fn test_ll_push_many() {
    let mut ll = LinkedList::new();
    for i in 0..10_000 {
        ll.push_front(i);
    }
    for i in 0..10_000 {
        println!("i={i}");
        assert!(ll.contains(&i))
    }
}

#[test]
fn test_ll_iter() {
    let mut ll = LinkedList::new();
    for i in 0..10 {
        ll.push_back(i);
    }
    for i in ll.iter() {
        println!("i in ll: {i}")
    }
    for (li, i) in ll.iter().zip(0..10) {
        assert_eq!(*li, i)
    }
}

#[test]
fn test_ll_iter_mut() {
    let mut ll = LinkedList::new();
    for i in 0..10 {
        ll.push_back(i);
    }
    for i in ll.iter_mut() {
        *i += 100;
    }
    for i in ll.iter() {
        println!("i in ll: {i}")
    }
    for (li, i) in ll.iter().zip(0..10) {
        assert_eq!(*li, 100 + i)
    }
}

#[test]
fn test_ll_iter_into() {
    let mut ll = LinkedList::new();
    for i in 0..10 {
        ll.push_back(i);
    }
    for i in ll.iter_mut() {
        *i += 100;
    }
    for i in ll.iter() {
        println!("i in ll: {i}")
    }
    for (li, i) in ll.into_iter().zip(0..10) {
        assert_eq!(li, 100 + i)
    }
}

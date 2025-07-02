use datastructurs::btree::BTreeSet;

fn wait_user() {
    println!("Press enter to continue");
    (std::io::stdin().read_line(&mut String::new())).unwrap();
}

fn main() {
    let mut tree = BTreeSet::new(3);
    for i in 0..20 {
        tree.insert(i);
        assert!(tree.contains(&i));
        println!("{tree}");
        wait_user();
    }

    for i in [18, 2, 5, 8, 11, 14] {
        println!("now removing {i}...");
        wait_user();
        tree.remove(&i);
        assert!(!tree.contains(&i));
        println!("{tree}");
    }

    println!("{tree}");
}

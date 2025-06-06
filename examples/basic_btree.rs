use datastructurs::btree::BTree;

fn main() {
    let mut tree = BTree::new(3);
    for i in 0..7 {
        tree.insert(i);
        assert!(tree.contains(&i));
        println!("{tree:#?}");
        println!("Press enter to continue");
        (std::io::stdin().read_line(&mut String::new())).unwrap();
    }

    tree.remove(&18);
    assert!(!tree.contains(&18));
    println!("{tree:#?}")
}

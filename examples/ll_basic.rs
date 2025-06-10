use datastructurs::linked_list::LinkedList;

fn main() {
    let mut v = LinkedList::new();
    for i in 0..50_000 {
        v.push_back(i);
    }
    for _ in v.iter() {}
    for i in v.into_iter() {
        println!("i:{i}")
    }
}

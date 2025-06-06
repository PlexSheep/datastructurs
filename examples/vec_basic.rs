use datastructurs::vec::Vec;

fn main() {
    let mut v = Vec::new();
    for i in 0..50_000 {
        v.push(i);
    }
    drop(v)
}

use datastructurs::{
    intrusive_linked_list::{IntrusiveList, ListLink},
    vec::Vec,
};
use datastructurs_macros::IntoIntrusiveList;

#[derive(Debug, IntoIntrusiveList)]
struct FooBar {
    #[allow(unused)]
    aaa: i32,
    #[accessor(AccA)]
    link_a: ListLink,
}

type List = IntrusiveList<FooBar, AccA>;

impl FooBar {
    fn new(aaa: i32) -> Self {
        FooBar {
            aaa,
            link_a: Default::default(),
        }
    }
}

fn main() {
    let mut a = FooBar::new(12);
    let mut store = Vec::new();
    let mut list = List::new();
    list.push_back(&mut a);

    for i in 0..32 {
        store.push(FooBar::new(i as i32));
        list.push_back(&mut store[i]);
    }

    for your_mom in list.iter() {
        println!("{}", your_mom.aaa)
    }

    println!("{}", list.debug_nodes())
}

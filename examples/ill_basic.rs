use datastructurs::{
    intrusive_linked_list::{IntrusiveList, ListLink},
    stable_ref::StableRefMut,
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
    let a = FooBar::new(12);
    let mut store = Vec::new();
    let mut list = List::new();
    list.push_back(StableRefMut::create_box(a));

    for i in 0..32 {
        store.push(FooBar::new(i as i32));
        // NOTE: the user needs to guarantee that the address of the data will never change if the data
        // is stored in some other datastructure.
        // TODO: is this example actually working fine?
        list.push_back(unsafe { StableRefMut::from_ref_to_raw(&mut store[i]) });
    }

    for your_mom in list.iter() {
        println!("{}", your_mom.aaa)
    }

    println!("{}", list.debug_nodes())
}

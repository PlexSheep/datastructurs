use std::mem::offset_of;

use crate::intrusive_linked_list::{
    IntoIntrusiveList, IntrusiveList, IntrusiveListAccessor, ListLink,
};
use crate::{vec, vec::Vec};

#[test]
fn test_ill_manual_impl_basic() {
    #[derive(PartialEq, Debug)]
    struct Foo {
        data: i32,
        name: String,
        link: ListLink,
    }
    impl Foo {
        fn new(id: i32) -> Self {
            Foo {
                data: id,
                name: format!("Foo{id}"),
                link: Default::default(),
            }
        }
    }
    struct FooAcc;
    impl IntrusiveListAccessor<Foo> for FooAcc {
        fn get_node(item: &Foo) -> &ListLink {
            &item.link
        }

        fn get_node_mut(item: &mut Foo) -> &mut ListLink {
            &mut item.link
        }

        unsafe fn from_node(node: &ListLink) -> &Foo {
            let offset = offset_of!(Foo, link);
            let p_node = node as *const _ as *const u8;
            let p_struct = unsafe { p_node.sub(offset) } as *const Foo;
            unsafe { &*p_struct }
        }

        unsafe fn from_node_mut(node: &mut ListLink) -> &mut Foo {
            let offset = offset_of!(Foo, link);
            let p_node = node as *const _ as *const u8;
            let p_struct = unsafe { p_node.sub(offset) } as *mut Foo;
            unsafe { &mut *p_struct }
        }
    }
    // impls done

    type List = IntrusiveList<Foo, FooAcc>;
    let mut list = List::new();
    let mut foos = vec![];
    for i in 0..19 {
        foos.push(Foo::new(i));
    }
    for foo in foos.iter_mut().step_by(2) {
        list.push_back(foo);
    }
    for foo in foos.iter_mut().skip(1).step_by(2) {
        list.push_back(foo);
    }
    for foo in foos.iter() {
        assert!(list.contains(foo))
    }
    println!("{}", list.debug_nodes());
    let elem_to_remove = &mut foos[5];
    dbg!(&elem_to_remove);
    dbg!(elem_to_remove.link.is_linked());
    assert!(list.contains(elem_to_remove));
    list.remove(elem_to_remove);
    assert!(!list.contains(elem_to_remove))
}

#[test]
fn test_ill_manual_impl_proc_macro() {
    #[derive(PartialEq, Debug, IntoIntrusiveList)]
    struct Foo {
        data: i32,
        name: String,
        #[accessor(FooAcc)]
        link: ListLink,
    }
    impl Foo {
        fn new(id: i32) -> Self {
            Foo {
                data: id,
                name: format!("Foo{id}"),
                link: Default::default(),
            }
        }
    }

    type List = IntrusiveList<Foo, FooAcc>;

    let mut list = List::new();
    let mut foos = vec![];
    for i in 0..19 {
        foos.push(Foo::new(i));
    }
    for foo in foos.iter_mut().step_by(2) {
        list.push_back(foo);
    }
    for foo in foos.iter_mut().skip(1).step_by(2) {
        list.push_back(foo);
    }
    for foo in foos.iter() {
        assert!(list.contains(foo))
    }
    println!("{}", list.debug_nodes());
    let elem_to_remove = &mut foos[5];
    dbg!(&elem_to_remove);
    dbg!(elem_to_remove.link.is_linked());
    assert!(list.contains(elem_to_remove));
    list.remove(elem_to_remove);
    assert!(!list.contains(elem_to_remove))
}

#[test]
fn test_ill_basic_derive() {
    #[derive(PartialEq, Debug, IntoIntrusiveList)]
    struct Bla {
        bi: f32,
        #[accessor(Blac)]
        link: ListLink,
    }
    type List = IntrusiveList<Bla, Blac>;
    let mut list = List::new();
    let mut datastore = Vec::new();
    for i in 0..22 {
        let bi = i as f32;
        let bla = Bla {
            bi,
            link: Default::default(),
        };
        datastore.push(bla);
        list.push_back(&mut datastore[i]);
    }

    dbg!(&datastore);
    println!("{}", list.debug_nodes());

    for i in 0..22 {
        assert_eq!(list[i].bi, i as f32)
    }
}

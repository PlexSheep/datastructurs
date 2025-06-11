use datastructurs::intrusive_linked_list::{IntoIntrusiveList, IntrusiveList, ListLink};
use datastructurs::stable_ref::StableRefMut;
use datastructurs::trace;
use datastructurs::vec::Vec;

#[derive(PartialEq, Debug, IntoIntrusiveList, Default, Clone, Copy)]
struct Bla {
    bi: f32,
    #[accessor(BlaAccessor)]
    link: ListLink,
}

impl Bla {
    fn new(bi: f32) -> Self {
        Self {
            bi,
            link: Default::default(),
        }
    }
}
type List = IntrusiveList<Bla, BlaAccessor>;

#[test]
fn test_ill_basic_derive() {
    let mut list = List::new();
    // NOTE: reserve enough capacity to make sure that the
    // elements are not moved in memory (reallocated).
    // If we do not explicitly do this, the addresses of the ListLinks become dandling when the
    // vector is reallocated to have more capacity
    let mut datastore = Vec::with_capacity(22);
    for i in 0..22 {
        let bla = Bla::new(i as f32);
        datastore.push(bla);
        let stable = unsafe { StableRefMut::from_ref_to_raw(&mut datastore[i]) };
        list.push_back(stable);
    }

    trace!("{}", list.debug_nodes());
    for i in 0..22 {
        assert_eq!(list[i].bi, i as f32)
    }
}

#[test]
fn test_ill_move_elements() {
    let mut list = List::new();
    let mut datastore = Vec::with_capacity(0);
    for i in 0..8 {
        datastore.push(Bla::new(i as f32));
        trace!("adding {:?} to list", datastore.last_mut().unwrap());
        let stable = unsafe { StableRefMut::from_ref_to_raw(datastore.last_mut().unwrap()) };
        list.push_back(stable);
        trace!("{}", list.debug_nodes());
    }
    trace!("{}", list.debug_nodes());
}

#[test]
fn test_ill_drop_elements() {
    let mut list = List::new();
    let mut datastore = Vec::with_capacity(0);
    for i in 0..8 {
        datastore.push(Bla::new(i as f32));
        trace!("adding {:?} to list", datastore.last_mut().unwrap());
        let stable = unsafe { StableRefMut::from_ref_to_raw(datastore.last_mut().unwrap()) };
        list.push_back(stable);
        trace!("{}", list.debug_nodes());
    }
    datastore.clear();
    assert!(list.is_empty()); // without explicit clear
    trace!("{}", list.debug_nodes());
}

use datastructurs::intrusive_linked_list::{
    IntoIntrusiveList, IntrusiveList, IntrusiveListAccessor, ListLink,
};
use datastructurs::trace;
use datastructurs::vec::Vec;

// BUG: this tests succeeds with --nocapture for some reason???
#[test]
fn test_ill_basic_derive() {
    #[derive(PartialEq, Debug, IntoIntrusiveList)]
    struct Bla {
        bi: f32,
        #[accessor(BlaAccessor)]
        link: ListLink,
    }
    type List = IntrusiveList<Bla, BlaAccessor>;

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
    trace!("{}", list.debug_nodes());
    for i in 0..22 {
        assert_eq!(list[i].bi, i as f32)
    }
}

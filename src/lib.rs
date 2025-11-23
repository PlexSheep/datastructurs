pub mod btree;
pub mod intrusive_linked_list;
pub mod linked_list;
pub mod raw_vec;
pub mod stable_ref;
pub mod sync;
pub mod vec;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! trace {
    ($($stuff:tt)+) => {
        println!("datastructu_rs::{}::{}: {}", file!(), line!(),format_args!($($stuff)+))
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! trace {
    ($($stuff:tt)+) => {
        () // ignore logs
    };
}

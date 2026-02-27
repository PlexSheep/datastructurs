pub mod btree;
pub mod linked_list;
pub mod raw_vec;
pub mod stable_ref;
pub mod sync;
pub mod vec;

#[macro_export]
macro_rules! trace_current_function {
    ($($stuff:tt)+) => {
        println!("datastructu_rs::{}::{}: {}", file!(), line!(),format_args!($($stuff)+))
    };
}

mod map;
mod set;
pub use map::BTreeMap;
pub use set::BTreeSet;

pub const DEFAULT_BRANCH_FACTOR: usize = 100;

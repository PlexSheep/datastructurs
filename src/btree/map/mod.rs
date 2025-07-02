use crate::btree::BTreeSet;

mod impls;

#[derive(Debug, Clone)]
struct MapPair<K: Ord + Copy, V: Clone> {
    key: K,
    value: V,
}

pub struct BTreeMap<K: Ord + Copy, V: Clone> {
    set: BTreeSet<MapPair<K, V>>,
}

#[cfg(test)]
mod tests;

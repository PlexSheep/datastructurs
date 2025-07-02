use std::fmt::{Debug, Display};

use crate::btree::{BTreeMap, map::MapPair};

impl<K: PartialEq, V> PartialEq for MapPair<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K: Eq, V> Eq for MapPair<K, V> {}

impl<K: PartialOrd, V> PartialOrd for MapPair<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K: Ord, V: Clone> Ord for MapPair<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: Display, V: Display> Display for MapPair<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

impl<K: Ord + Clone + Display + Debug, V: Display + Clone + Debug> Display for BTreeMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.set, f)
    }
}

impl<K: Ord + Clone + Debug, V: Debug + Clone> Debug for BTreeMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.set, f)
    }
}

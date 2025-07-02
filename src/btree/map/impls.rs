use crate::btree::map::MapPair;

impl<K: PartialEq + Ord, V: Clone> PartialEq for MapPair<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K: Eq + Ord, V: Clone> Eq for MapPair<K, V> {}

impl<K: PartialOrd + Ord, V: Clone> PartialOrd for MapPair<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, V: Clone> Ord for MapPair<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

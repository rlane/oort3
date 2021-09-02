pub use rapier2d_f64::data::arena::Index;
use std::collections::HashMap;

pub trait HasIndex {
    fn index(self) -> Index;
}

pub struct IndexSet<T: HasIndex> {
    indices: Vec<T>,
    positions: HashMap<T, usize>,
}

impl<T: HasIndex + Eq + std::hash::Hash + Copy> IndexSet<T> {
    pub fn new() -> Self {
        IndexSet::<T> {
            indices: Vec::new(),
            positions: HashMap::new(),
        }
    }

    pub fn insert(self: &mut IndexSet<T>, handle: T) {
        self.indices.push(handle);
        self.positions.insert(handle, self.indices.len() - 1);
    }

    pub fn remove(self: &mut IndexSet<T>, handle: T) {
        let pos = self.positions[&handle];
        self.indices[pos] = self.indices[self.indices.len() - 1];
        self.positions.insert(self.indices[pos], pos);
        self.positions.remove(&handle);
        self.indices.pop();
    }

    pub fn contains(self: &IndexSet<T>, handle: T) -> bool {
        self.positions.contains_key(&handle)
    }

    pub fn iter(self: &IndexSet<T>) -> std::slice::Iter<T> {
        self.indices.iter()
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

impl<T: HasIndex + Eq + std::hash::Hash + Copy> Default for IndexSet<T> {
    fn default() -> IndexSet<T> {
        IndexSet::new()
    }
}

#[cfg(test)]
mod test {
    use super::{HasIndex, Index, IndexSet};
    use test_env_log::test;

    fn list<T: HasIndex + Eq + std::hash::Hash + Copy>(index_set: &IndexSet<T>) -> Vec<T> {
        index_set.iter().copied().collect::<Vec<T>>()
    }

    #[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
    pub struct TestHandle(pub Index);

    impl HasIndex for TestHandle {
        fn index(self) -> Index {
            self.0
        }
    }

    #[test]
    fn test_index_set() {
        let mut index_set: IndexSet<TestHandle> = IndexSet::new();
        let handle0 = TestHandle(Index::from_raw_parts(2, 1));
        let handle1 = TestHandle(Index::from_raw_parts(1, 20));

        assert_eq!(list(&index_set), vec![]);

        index_set.insert(handle0);
        assert_eq!(list(&index_set), vec![handle0]);

        index_set.insert(handle1);
        assert_eq!(list(&index_set), vec![handle0, handle1]);

        index_set.remove(handle0);
        assert_eq!(list(&index_set), vec![handle1]);

        index_set.remove(handle1);
        assert_eq!(list(&index_set), vec![]);
    }
}

use rapier2d_f64::data::arena::Index;
use std::collections::HashMap;

pub struct IndexSet {
    indices: Vec<Index>,
    positions: HashMap<Index, usize>,
}

impl IndexSet {
    pub fn new() -> IndexSet {
        IndexSet {
            indices: Vec::new(),
            positions: HashMap::new(),
        }
    }

    pub fn insert(self: &mut IndexSet, index: Index) {
        self.indices.push(index);
        self.positions.insert(index, self.indices.len() - 1);
    }

    pub fn remove(self: &mut IndexSet, index: Index) {
        let pos = self.positions[&index];
        self.indices[pos] = self.indices[self.indices.len() - 1];
        self.positions.insert(self.indices[pos], pos);
        self.positions.remove(&index);
        self.indices.pop();
    }

    pub fn iter(self: &IndexSet) -> std::slice::Iter<Index> {
        self.indices.iter()
    }
}

impl Default for IndexSet {
    fn default() -> IndexSet {
        IndexSet::new()
    }
}

#[cfg(test)]
fn list(index_set: &IndexSet) -> Vec<Index> {
    index_set.iter().copied().collect::<Vec<Index>>()
}

#[test]
fn test_index_set() {
    let mut index_set = IndexSet::new();
    let idx0 = Index::from_raw_parts(2, 1);
    let idx1 = Index::from_raw_parts(1, 20);

    assert_eq!(list(&index_set), vec![]);

    index_set.insert(idx0);
    assert_eq!(list(&index_set), vec![idx0]);

    index_set.insert(idx1);
    assert_eq!(list(&index_set), vec![idx0, idx1]);

    index_set.remove(idx0);
    assert_eq!(list(&index_set), vec![idx1]);

    index_set.remove(idx1);
    assert_eq!(list(&index_set), vec![]);
}

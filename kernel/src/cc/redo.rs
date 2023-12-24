extern crate alloc;
use alloc::vec::Vec;
pub struct RedoEntry<K: Eq, V> {
    pub key: K,
    pub val: V,
}

pub struct RedoLog<K:Eq + Clone, V: Clone> {
    pub entries: Vec<RedoEntry<K, V>>,
}

impl<K: Eq + Clone, V: Clone> RedoLog<K, V> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.entries.iter().find(|e| &e.key == key).map(|e| e.val.clone())
    }

    pub fn insert(&mut self, key: &K, val: &V) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.key == *key) {
            entry.val = val.clone();
        } else {
            self.entries.push(RedoEntry { key:key.clone(), val:val.clone() });
        }
    }

    pub fn update(&mut self, key: &K, new_val: &V) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.key == *key) {
            entry.val = new_val.clone();
        }
    }

    pub fn remove(&mut self, key: &K) {
        if let Some(index) = self.entries.iter().position(|e| &e.key == key) {
            self.entries.swap_remove(index);
        }
    }

    pub fn size(&self) -> usize{
        self.entries.len()
    }
}

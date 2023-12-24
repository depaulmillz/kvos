extern crate alloc;
use alloc::vec::Vec;
use spin::mutex::Mutex;
use crate::common::hash::Mix13Hash;
pub trait Map {
    type Key;
    type Value;
    fn new() -> Self;
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn insert(&self, key: &Self::Key, value: &Self::Value) -> bool;
    fn remove(&self, key: &Self::Key) -> bool;
}

struct Entry<K, V> {
    key: K,
    value: V,
}

struct Bucket<K, V> {
    data: Mutex<Vec<Entry<K, V>>>,
}

pub struct SimpleHashMap<K, V> {
    buckets: Vec<Bucket<K, V>>,
    num_buckets: usize,
}
impl <K: Eq + core::hash::Hash + AsRef<[u8]>, V: Clone>  SimpleHashMap<K, V> {
    pub fn with_capacity(num_buckets:usize) -> Self {
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(Bucket {
                data: Mutex::new(Vec::new()),
            });
        }

        SimpleHashMap { buckets, num_buckets }
    } 
}
impl<K: Eq + Clone + core::hash::Hash + AsRef<[u8]>, V: Clone> Map for SimpleHashMap<K, V> {
    type Key = K;
    type Value = V;

    fn new() -> Self {
        let num_buckets = 16; // Default number of buckets
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(Bucket {
                data: Mutex::new(Vec::new()),
            });
        }

        SimpleHashMap { buckets, num_buckets }
    }

    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        let bucket_idx = self.compute_bucket_idx(key);
        self.buckets[bucket_idx].data.lock().iter().find(|e| &e.key == key).map(|e| e.value.clone())
    }

    fn insert(&self, key: &Self::Key, value: &Self::Value) -> bool {
        let bucket_idx = self.compute_bucket_idx(key);
        let bucket = &self.buckets[bucket_idx];
        let mut bucket_data = bucket.data.lock();
        if let Some(entry) = bucket_data.iter_mut().find(|e| &e.key == key) {
            entry.value = value.clone();
            true
        } else {
            bucket_data.push(Entry {
                key: key.clone(),
                value: value.clone(),
            });
            true
        }
    }

    fn remove(&self, key: &Self::Key) -> bool {
        let bucket_idx = self.compute_bucket_idx(key);
        let bucket = &self.buckets[bucket_idx];
        let mut bucket_data = bucket.data.lock();
        if let Some(pos) = bucket_data.iter().position(|e| &e.key == key) {
            bucket_data.remove(pos);
            true
        } else {
            false
        }
    }
}

impl<K: Eq + core::hash::Hash + AsRef<[u8]>, V> SimpleHashMap<K, V> {
    fn compute_bucket_idx(&self, key: &K) -> usize {
        let hash = Mix13Hash::new().compute_hash(&key.as_ref().to_vec());
        (hash % self.num_buckets as u64) as usize
    }
}

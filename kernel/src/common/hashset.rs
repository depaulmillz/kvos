use alloc::vec::Vec;
use crate::common::hash::Mix13Hash;

pub trait Set {
    type Key;
    fn new() -> Self;
    fn contains(&self, key: &Self::Key) -> bool;
    fn insert(&mut self, key: &Self::Key) -> bool;
    fn remove(&mut self, key: &Self::Key) -> bool;
    fn size(&self)->usize;
}

pub struct Bucket<K> {
    pub data: Vec<K>,
}

pub struct SimpleHashSet<K> {
    buckets: Vec<Bucket<K>>,
    num_buckets: usize,
}

impl<K: Eq + core::hash::Hash> SimpleHashSet<K> {
    pub fn with_capacity(num_buckets: usize) -> Self {
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(Bucket {
                data: Vec::new(),
            });
        }

        SimpleHashSet { buckets, num_buckets }
    }

    fn compute_bucket_idx(&self, key: &K) -> usize {
        let hash = Mix13Hash::new().compute_hash(key.as_ref.to_vec());
        (hash % self.num_buckets as u64) as usize
    }
}

impl<K: Eq + Clone + core::hash::Hash> Set for SimpleHashSet<K> {
    type Key = K;

    fn new() -> Self {
        let num_buckets = 16; // Default number of buckets
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(Bucket {
                data: Vec::new(),
            });
        }

        SimpleHashSet { buckets, num_buckets }
    }

    fn contains(&self, key: &Self::Key) -> bool {
        let bucket_idx = self.compute_bucket_idx(key);
        self.buckets[bucket_idx].data.contains(key)
    }

    fn insert(&mut self, key: &Self::Key) -> bool {
        let bucket_idx = self.compute_bucket_idx(key);
        let bucket = &mut self.buckets[bucket_idx];
        if !bucket.data.contains(key) {
            bucket.data.push(key.clone());
            true
        } else {
            false
        }
    }

    fn remove(&mut self, key: &Self::Key) -> bool {
        let bucket_idx = self.compute_bucket_idx(key);
        let bucket = &mut self.buckets[bucket_idx];
        if let Some(pos) = bucket.data.iter().position(|k| k == key) {
            bucket.data.remove(pos);
            true
        } else {
            false
        }
    }

    fn size(&self) -> usize {
        self.buckets.iter().map(|bucket| bucket.data.len()).sum()
    }
}

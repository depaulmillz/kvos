pub mod redo;
extern crate alloc;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};
use redo::RedoLog;
use crate::common::locktable::{LockTable, TryLockResult};
use crate::common::set::SimpleSet;
use crate::common::hash::Mix13Hash;
use crate::common::map::Map;
use crate::disk::persistentmap::PersistentMap;
use lazy_static::lazy_static;
use core::cell::RefCell;



pub fn getpid()->i32 {
    0
}
pub trait Transaction {
    type Key;

    type Value;

    // type Map: Map<Key = Self::Key, Value = Self::Value>;
    fn read(&mut self, key: &Self::Key) -> Option<Self::Value>;

    fn write(&mut self, key: &Self::Key, value: &Self::Value);

    fn delete(&mut self, key: &Self::Key) -> Option<bool>;

    fn abort(&self);

    fn try_commit(&self) -> bool;

    fn try_commit_persist(&self) -> bool;
}

struct Counter {
    value: AtomicU64,
}

impl Counter {
    fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    fn increment(&self) ->u64 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }
}

lazy_static! {
    static ref GLOBAL_CLOCK: Counter= Counter::new();
}

pub struct WDTX<K:Eq + core::hash::Hash + Clone, V: Clone, M: Map>
{
    tid: u64,
    locked_index: SimpleSet<u64>,
    redo_log: RedoLog<K, V>,
    remove_log: SimpleSet<K>,
    lock_table: Arc<LockTable>,
    map: Arc<M>,
    is_aborted: bool,
    hash: RefCell<Mix13Hash>,
}



impl<K:Eq + core::hash::Hash + Clone + AsRef<[u8]>, V: Clone, M: Map> WDTX<K, V, M> {
    pub fn new(_lock_table: Arc<LockTable>, _map: Arc<M>) -> Self {
        let _pid = getpid();
        let _tid= GLOBAL_CLOCK.increment();
        let mut _hash = Mix13Hash::new();
        let mut _locked_index = SimpleSet::new();
        let mut _redo_log = RedoLog::new();
        let mut _remove_log = SimpleSet::new();

        WDTX {
            tid: _tid,
            locked_index : _locked_index,
            redo_log : _redo_log,
            remove_log : _remove_log,
            lock_table : _lock_table,
            map : _map,
            is_aborted : false,
            hash: RefCell::new(Mix13Hash::new()),
        }
    }
    fn hash(&self, key:&K, size:u64) -> u64{
        let hasher = self.hash.borrow_mut();
        let val = hasher.compute_hash(&key.as_ref().to_vec());
        val % size
    }
}



impl<K:Eq + core::hash::Hash + Clone + AsRef<[u8]>, V: Clone, M:Map<Key = K, Value = V>> Transaction for WDTX<K, V, M> {
    type Key = K;
    type Value = V;
    fn read(&mut self,key: &Self::Key) -> Option<Self::Value> {
        if self.is_aborted {
            return None;
        }
        if let Some(v) = self.redo_log.get(key) {
            return Some(v);
        }
    
        if self.locked_index.contains(&self.hash(key, self.lock_table.size())) {
            if self.remove_log.contains(key) {
                return None;
            }
            return self.map.get(key);
        }
        
        let hash_value = self.hash(key, self.lock_table.size());
        let ret = self.lock_table.try_lock(hash_value, self.tid);
        loop {
            if ret == TryLockResult::Success {
                self.locked_index.insert(&self.hash(key, self.lock_table.size()));
                return self.map.get(key);
            } else if ret == TryLockResult::Wait {
            } else {
                self.is_aborted = true;
                self.abort();
                return None;
            }
        }
    }

    fn write(&mut self, key: &Self::Key, value: &Self::Value) {
        if self.is_aborted {
            return;
        }
        if let Some(_) = self.redo_log.get(key) {
            self.redo_log.update(key, value);
            return;
        }

        if !self.locked_index.contains(&self.hash(key, self.lock_table.size())) {
            let hash_value = self.hash(key, self.lock_table.size());
            let ret = self.lock_table.try_lock(hash_value, self.tid);
            // let ret = self.lock_table.borrow_mut().try_lock(self.hash(key, self.lock_table.borrow().size()), self.tid);
            loop {
                if ret == TryLockResult::Success {
                    self.locked_index.insert(&self.hash(key, self.lock_table.size()));
                    break;
                } else if ret == TryLockResult::Wait {
                } else {
                    self.is_aborted = true;
                    self.abort();
                    return;
                }
            }
        }
        self.redo_log.insert(key, value);
    }

    fn delete(&mut self, key: &Self::Key) -> Option<bool>{
        if self.is_aborted {
            return None;
        }
        if self.remove_log.contains(key) == true {
            return Some(false);
        }
        if let Some(_) = self.redo_log.get(key) {
            self.redo_log.remove(key);
            self.remove_log.insert(key);
            return Some(true);
        }

        if !self.locked_index.contains(&self.hash(key, self.lock_table.size())) {
            let ret = self.lock_table.try_lock(self.hash(key, self.lock_table.size()), self.tid);
            loop {
                if ret == TryLockResult::Success {
                    self.locked_index.insert(&self.hash(key, self.lock_table.size()));
                    break;
                } else if ret == TryLockResult::Wait {
                } else {
                    self.is_aborted = true;
                    return None;
                }
                
            }
        }
        if let Some(_) = self.map.get(key) {
            self.remove_log.insert(key);
            return Some(true);
        }
        return Some(false);
        
    }

    fn abort(&self) {
        for key in &self.locked_index.data {
            self.lock_table.unlock(*key);
        }
    }

    fn try_commit(&self) -> bool{
        if self.is_aborted {
            return false;
        }
        
        for entry in &self.redo_log.entries {
            self.map.insert(&entry.key, &entry.val.clone());
        }
        for key in &self.remove_log.data {
            self.map.remove(key);
        }
        for key in &self.locked_index.data {
            self.lock_table.unlock(*key);
        }
        true
    }

    fn try_commit_persist(&self) -> bool {
        false
    }
}
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct WDTXPersist<K:Eq + core::hash::Hash + Clone, V: Clone, M: PersistentMap>
{
    tid: u64,
    locked_index: SimpleSet<u64>,
    redo_log: RedoLog<K, V>,
    remove_log: SimpleSet<K>,
    lock_table: Arc<LockTable>,
    map: Arc<M>,
    is_aborted: bool,
    hash: RefCell<Mix13Hash>,
}

impl<K:Eq + core::hash::Hash + Clone + AsRef<[u8]>, V: Clone, M: PersistentMap> WDTXPersist<K, V, M> {
    pub fn new(_lock_table: Arc<LockTable>, _map: Arc<M>) -> Self {
        let _pid = getpid();
        let _tid= GLOBAL_CLOCK.increment();
        let mut _hash = Mix13Hash::new();
        let mut _locked_index = SimpleSet::new();
        let mut _redo_log = RedoLog::new();
        let mut _remove_log = SimpleSet::new();

        WDTXPersist {
            tid: _tid,
            locked_index : _locked_index,
            redo_log : _redo_log,
            remove_log : _remove_log,
            lock_table : _lock_table,
            map : _map,
            is_aborted : false,
            hash: RefCell::new(Mix13Hash::new()),
        }
    }
    fn hash(&self, key:&K, size:u64) -> u64{
        let hasher = self.hash.borrow_mut();
        let val = hasher.compute_hash(&key.as_ref().to_vec());
        val % size
    }
}

impl<K:Eq + core::hash::Hash + Clone + AsRef<[u8]>, V: Clone, M:PersistentMap<Key = K, Value = V>> Transaction for WDTXPersist<K, V, M> {
    type Key = K;
    type Value = V;
    fn read(&mut self,key: &Self::Key) -> Option<Self::Value> {
        if self.is_aborted {
            return None;
        }

        if let Some(v) = self.redo_log.get(key) {
            return Some(v);
        }
    
        if self.locked_index.contains(&self.hash(key, self.lock_table.size())) {
            if self.remove_log.contains(key) {
                return None;
            }
            return self.map.get(key);
        }
        
        let hash_value = self.hash(key, self.lock_table.size());
        let ret = self.lock_table.try_lock(hash_value, self.tid);
        loop {
            if ret == TryLockResult::Success {
                self.locked_index.insert(&self.hash(key, self.lock_table.size()));
                return self.map.get(key);
            } else if ret == TryLockResult::Wait {
            } else {
                self.is_aborted = true;
                self.abort();
                return None;
            }
        }
    }

    fn write(&mut self, key: &Self::Key, value: &Self::Value) {
        if self.is_aborted {
            return;
        }
        if let Some(_) = self.redo_log.get(key) {
            self.redo_log.update(key, value);
            return;
        }

        if !self.locked_index.contains(&self.hash(key, self.lock_table.size())) {
            let hash_value = self.hash(key, self.lock_table.size());
            let ret = self.lock_table.try_lock(hash_value, self.tid);
            // let ret = self.lock_table.borrow_mut().try_lock(self.hash(key, self.lock_table.borrow().size()), self.tid);
            loop {
                if ret == TryLockResult::Success {
                    self.locked_index.insert(&self.hash(key, self.lock_table.size()));
                    break;
                } else if ret == TryLockResult::Wait {
                } else {
                    self.is_aborted = true;
                    self.abort();
                    return;
                }
            }
        }
        self.redo_log.insert(key, value);
    }

    fn delete(&mut self, key: &Self::Key) -> Option<bool>{
        if self.is_aborted {
            return None;
        }
        if self.remove_log.contains(key) == true {
            return Some(false);
        }
        if let Some(_) = self.redo_log.get(key) {
            self.redo_log.remove(key);
            self.remove_log.insert(key);
            return Some(true);
        }

        if !self.locked_index.contains(&self.hash(key, self.lock_table.size())) {
            let ret = self.lock_table.try_lock(self.hash(key, self.lock_table.size()), self.tid);
            loop {
                if ret == TryLockResult::Success {
                    self.locked_index.insert(&self.hash(key, self.lock_table.size()));
                    break;
                } else if ret == TryLockResult::Wait {
                } else {
                    self.is_aborted = true;
                    return None;
                }
                
            }
        }
        if let Some(_) = self.map.get(key) {
            self.remove_log.insert(key);
            return Some(true);
        }
        return Some(false);
        
    }

    fn abort(&self) {
        for key in &self.locked_index.data {
            self.lock_table.unlock(*key);
        }
    }

    fn try_commit(&self) -> bool{
        if self.is_aborted {
            return false;
        }
        
        for entry in &self.redo_log.entries {
            let _ = self.map.insert_no_log(&entry.key, &entry.val.clone());
        }
        for key in &self.remove_log.data {
            let _ = self.map.remove_no_log(key);
        }
        for key in &self.locked_index.data {
            self.lock_table.unlock(*key);
        }
        true
    }

    fn try_commit_persist(&self) -> bool{
        if self.is_aborted {
            return false;
        }
        
        for entry in &self.redo_log.entries {
            let _ = self.map.insert(&entry.key, &entry.val.clone());
        }
        for key in &self.remove_log.data {
            let _ = self.map.remove(key);
        }
        for key in &self.locked_index.data {
            self.lock_table.unlock(*key);
        }
        true
    }
}

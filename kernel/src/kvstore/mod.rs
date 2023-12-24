use crate::cc::{Transaction, WDTXPersist, WDTX};
use crate::common::{locktable::LockTable, map::SimpleHashMap};
use crate::disk::persistentmap::PersistentMap;
use crate::disk::persistentmap::{PersistentHashMap, ToBeBytes};
extern crate alloc;
use alloc::sync::Arc;
pub trait KVStore {
    type Key;
    type Value;
    type Transaction;
    fn begin(&self) -> Self::Transaction;
    fn new(map_size: u64, locktable_size: u64) -> Arc<Self>;
    fn transact<F>(&self, f: F, persist: bool)
    where
        F: Fn(&mut Self::Transaction) -> ();

    fn transact_mut<F>(&self, f: &mut F, persist: bool)
    where
        F: FnMut(&mut Self::Transaction) -> ();
}
pub struct TxKVStore<K: Eq + core::hash::Hash + AsRef<[u8]>, V: Clone> {
    map: Arc<SimpleHashMap<K, V>>,
    lock_table: Arc<LockTable>,
}
impl<K: Eq + core::hash::Hash + Clone + AsRef<[u8]>, V: Clone> KVStore for TxKVStore<K, V> {
    type Key = K;
    type Value = V;
    type Transaction = WDTX<K, V, SimpleHashMap<K, V>>;
    fn begin(&self) -> Self::Transaction {
        WDTX::new(self.lock_table.clone(), self.map.clone())
    }
    fn new(map_size: u64, lock_table_size: u64) -> Arc<Self> {
        let map = Arc::new(SimpleHashMap::with_capacity(map_size as usize));
        let lock_table = Arc::new(LockTable::new(lock_table_size));
        Arc::new(Self {
            map: map,
            lock_table: lock_table,
        })
    }

    fn transact<F>(&self, f: F, _persist: bool)
    where
        F: Fn(&mut Self::Transaction) -> (),
    {
        let mut committed = false;
        while !committed {
            let mut tx = self.begin();
            f(&mut tx);
            committed = tx.try_commit();
            assert_eq!(committed, true);
        }
    }

    fn transact_mut<F>(&self, f: &mut F, _persist :bool)
    where
        F: FnMut(&mut Self::Transaction) -> (),
    {
        let mut committed = false;
        while !committed {
            let mut tx = self.begin();
            f(&mut tx);
            committed = tx.try_commit();
        }
    }
}

pub struct TxKVStorePersist<
    K: Eq + core::hash::Hash + AsRef<[u8]> + ToBeBytes,
    V: Clone + ToBeBytes,
> {
    map: Arc<PersistentHashMap<K, V>>,
    lock_table: Arc<LockTable>,
}
impl<K: Eq + core::hash::Hash + Clone + AsRef<[u8]> + ToBeBytes, V: Clone + ToBeBytes> KVStore
    for TxKVStorePersist<K, V>
{
    type Key = K;
    type Value = V;
    type Transaction = WDTXPersist<K, V, PersistentHashMap<K, V>>;
    fn begin(&self) -> Self::Transaction {
        WDTXPersist::new(self.lock_table.clone(), self.map.clone())
    }
    fn new(_map_size: u64, lock_table_size: u64) -> Arc<Self> {
        // let map = Arc::new(PersistentHashMap::with_capacity(map_size as usize));
        let map = Arc::new(PersistentHashMap::build_from_disk());
        let lock_table = Arc::new(LockTable::new(lock_table_size));
        Arc::new(Self {
            map: map,
            lock_table: lock_table,
        })
    }

    fn transact<F>(&self, f: F, persist: bool)
    where
        F: Fn(&mut Self::Transaction) -> (),
    {
        let mut committed = false;
        while !committed {
            let mut tx = self.begin();
            f(&mut tx);
            if persist {
                committed = tx.try_commit_persist();
            }else {
                committed = tx.try_commit();
            }
            
            assert_eq!(committed, true);
        }
    }

    fn transact_mut<F>(&self, f: &mut F, persist: bool)
    where
        F: FnMut(&mut Self::Transaction) -> (),
    {
        let mut committed = false;
        while !committed {
            let mut tx = self.begin();
            f(&mut tx);
            if persist {
                committed = tx.try_commit_persist();
            } else {
                committed = tx.try_commit();
            }
        }
    }
}

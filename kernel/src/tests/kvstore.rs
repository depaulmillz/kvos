use super::KernelTest;
use crate::serial_print;
use crate::cc::Transaction;
use crate::common::hash::{Mix13Hash,mix13};
use crate::common::locktable::{LockTable, TryLockResult};
use crate::common::map::{Map, SimpleHashMap};
use crate::common::set::SimpleSet;
use crate::cc::redo::RedoLog;
use crate::kvstore::{KVStore,TxKVStorePersist};
use crate::map::SkipMap;
use alloc::string::String;

/////////////////////////////////////////////////////////////
//// Tests
////////////////////////////////////////////////////////////


fn test_mix13hash() {
    let hasher = Mix13Hash::new();

    // Input data
    let data = b"Hello, world!";
    let result = hasher.compute_hash(data);

    // Compute expected value manually
    let mut expected = 0u64;
    for &byte in data {
        expected ^= (byte as u64) << ((expected % 8) * 8);
        expected = mix13(expected);
    }

    assert_eq!(
        result, expected,
        "Hash output did not match expected value."
    );

    let result2 = hasher.compute_hash(data);
    assert_eq!(
        result2, expected,
        "Hash output did not match expected value."
    );
}

fn test_lock_table_initialization() {
    let table = LockTable::new(10);
    assert_eq!(table.size(), 10, "LockTable size did not match expected value.");
}

fn test_try_lock_success() {
    let table = LockTable::new(10);
    let result = table.try_lock(5, 1);
    assert_eq!(result, TryLockResult::Success, "Expected to acquire the lock successfully.");
}

fn test_try_lock_die() {
    let table = LockTable::new(10);
    table.try_lock(5, 1);
    let result = table.try_lock(5, 0);
    assert_eq!(result, TryLockResult::Wait, "Expected to be Wait the lock due to older transaction ID.");
}

fn test_try_lock_wait() {
    let table = LockTable::new(10);
    table.try_lock(5, 1);
    let result = table.try_lock(5, 2);
    assert_eq!(result, TryLockResult::Die, "Expected to Die for the lock due to younger transaction ID.");
}

fn test_unlock() {
    let table = LockTable::new(10);
    table.try_lock(5, 1);
    table.unlock(5);
    let result = table.try_lock(5, 2);
    assert_eq!(result, TryLockResult::Success, "Expected to acquire the lock successfully after unlocking.");
}

fn test_simple_map_insert_and_get() {
    let map: SimpleHashMap<&str, i32> = SimpleHashMap::new();

    // Test insert
    assert!(map.insert(&"key1", &10));
    assert_eq!(map.get(&"key1"), Some(10));

    // Test update
    assert!(map.insert(&"key1", &20));
    assert_eq!(map.get(&"key1"), Some(20));
}

fn test_simple_map_remove() {
    let map: SimpleHashMap<&str, i32> = SimpleHashMap::new();
    map.insert(&"key1", &10);

    // Test remove existing key
    assert!(map.remove(&"key1"));
    assert_eq!(map.get(&"key1"), None);

    // Test remove non-existing key
    assert!(!map.remove(&"key1"));
}

fn test_simple_map_with_capacity() {
    let map: SimpleHashMap<&str, i32> = SimpleHashMap::with_capacity(2);

    // Test insert within capacity
    assert!(map.insert(&"key1", &10));
    assert!(map.insert(&"key2", &20));
    assert_eq!(map.get(&"key1"), Some(10));
    assert_eq!(map.get(&"key2"), Some(20));

    // Test insert beyond capacity
    assert!(map.insert(&"key3", &30));
    assert_eq!(map.get(&"key3"), Some(30));
}

fn test_set_insert() {
    let mut set = SimpleSet::<u32>::new();
    assert!(set.insert(&5));
    assert!(!set.insert(&5)); // Duplicate insert should return false
}

fn test_set_remove() {
    let mut set = SimpleSet::new();
    set.insert(&5);
    assert_eq!(set.remove(&5), true);  // Removing an existing item should return true.
    assert_eq!(set.remove(&5), false); // Removing a non-existent item should return false.
}


fn test_set_contains() {
    let mut set = SimpleSet::new();
    set.insert(&5);
    assert_eq!(set.contains(&5), true);  // SimpleSet contains the item.
    assert_eq!(set.contains(&10), false); // SimpleSet doesn't contain the item.
}

fn test_set_size() {
    let mut set = SimpleSet::new();
    assert_eq!(set.size(), 0); // Initial size should be 0.
    set.insert(&5);
    assert_eq!(set.size(), 1); // Size after inserting an item.
    set.insert(&10);
    assert_eq!(set.size(), 2); // Size after inserting another item.
    set.remove(&5);
    assert_eq!(set.size(), 1); // Size after removing an item.
}

fn test_redo_insert() {
    let mut log = RedoLog::new();
    log.insert(&"key1", &"value1");
    assert_eq!(log.get(&"key1"), Some("value1"));

    // Test overwriting existing key
    log.insert(&"key1", &"value2");
    assert_eq!(log.get(&"key1"), Some("value2"));
}

fn test_redo_update() {
    let mut log = RedoLog::new();
    log.insert(&"key1", &"value1");
    log.update(&"key1", &"updated_value1");
    assert_eq!(log.get(&"key1"), Some("updated_value1"));

    // Test updating non-existent key (shouldn't change anything)
    log.update(&"key2", &"value2");
    assert_eq!(log.get(&"key2"), None);
}

fn test_redo_remove() {
    let mut log = RedoLog::new();
    log.insert(&"key1", &"value1");
    log.remove(&"key1");
    assert_eq!(log.get(&"key1"), None);
}

fn test_redo_size() {
    let mut log = RedoLog::new();
    assert_eq!(log.size(), 0); // Initial size should be 0.
    log.insert(&"key1", &"value1");
    assert_eq!(log.size(), 1); // Size after inserting an entry.
    log.insert(&"key2", &"value2");
    assert_eq!(log.size(), 2); // Size after inserting another entry.
    log.remove(&"key1");
    assert_eq!(log.size(), 1); // Size after removing an entry.
}

fn test_tx() {
    let kv = TxKVStorePersist::new(32, 32);
    // let tx = kv.begin();
    kv.transact(|tx| {
        tx.write(&String::from("bootloader_name"), &String::from("crate_boot"));
        let name = tx.read(&String::from("bootloader_name"));
        assert_eq!(name, Some(String::from("crate_boot")));
    }, true);

    kv.transact(|tx| {
        let name = tx.read(&String::from("bootloader_name"));
        assert_eq!(name, Some(String::from("crate_boot")));
    }, true);
}

fn test_skipmap_insert_get() {
    let map: SkipMap<&str, i32> = SkipMap::new();

    // Test insert
    assert!(map.insert(&"key1", 10));
    assert!(map.insert(&"key2", 20));
    assert_eq!(map.get(&"key1"), Some(10));
    assert_eq!(map.get(&"key2"), Some(20));

    // Test update
    assert!(map.insert(&"key1", 20));
    assert_eq!(map.get(&"key1"), Some(20));
}

fn test_skipmap_remove() {
    let map: SkipMap<&str, i32> = SkipMap::new();
    assert!(map.insert(&"key1", 10));

    // Test remove existing key
    assert!(map.remove(&"key1"));
    assert_eq!(map.get(&"key1"), None);

    // Test remove non-existing key
    assert!(!map.remove(&"key1"));
}

pub fn run_tests() {
    let tests = [
        KernelTest {
            name : "test_mix13hash",
            test_fn : test_mix13hash,
        },
        KernelTest {
            name : "test_lock_table_initialization",
            test_fn : test_lock_table_initialization,
        },
        KernelTest {
            name : "test_try_lock_success",
            test_fn : test_try_lock_success,
        },
        KernelTest {
            name : "test_try_lock_die",
            test_fn : test_try_lock_die,
        },
        KernelTest {
            name : "test_try_lock_wait",
            test_fn : test_try_lock_wait,
        },
        KernelTest {
            name : "test_unlock",
            test_fn : test_unlock,
        },
        KernelTest {
            name : "test_simple_map_insert_and_get",
            test_fn : test_simple_map_insert_and_get,
        },
        KernelTest {
            name : "test_simple_map_remove",
            test_fn : test_simple_map_remove,
        },
        KernelTest {
            name : "test_simple_map_with_capacity",
            test_fn : test_simple_map_with_capacity,
        },
        KernelTest {
            name : "test_set_insert",
            test_fn : test_set_insert,
        },
        KernelTest {
            name : "test_set_remove",
            test_fn : test_set_remove,
        },
        KernelTest {
            name : "test_set_contains",
            test_fn : test_set_contains,
        },
        KernelTest {
            name : "test_set_size",
            test_fn : test_set_size,
        },
        KernelTest {
            name : "test_redo_insert",
            test_fn : test_redo_insert,
        },
        KernelTest {
            name : "test_redo_update",
            test_fn : test_redo_update,
        },
        KernelTest {
            name : "test_redo_remove",
            test_fn : test_redo_remove,
        },
        KernelTest {
            name : "test_redo_size",
            test_fn : test_redo_size,
        },
        KernelTest {
            name : "test_tx",
            test_fn : test_tx,
        },
        KernelTest {
            name : "test_skipmap_insert_get",
            test_fn : test_skipmap_insert_get,
        },
        KernelTest {
            name : "test_skipmap_remove",
            test_fn : test_skipmap_remove,
        },
    ];
    for t in tests.iter() {
        serial_print!("{}...\t", t.name);
        (t.test_fn)();
        serial_print!("[ok]\n");
    }
}

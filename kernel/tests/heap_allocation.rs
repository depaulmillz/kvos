#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kvos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use bootloader::{entry_point, BootInfo};
use kvos::cc::Transaction;
use core::panic::PanicInfo;
use kvos::allocator::HEAP_SIZE;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use kvos::allocator;
    use kvos::hlt_loop;

    kvos::init();

    allocator::init_heap(boot_info).expect("Heap allocation failed");

    test_main();

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kvos::test_panic_handler(info)
}

/////////////////////////////////////////////////////////////
//// Tests
////////////////////////////////////////////////////////////

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(42);
    let heap_value_2 = Box::new(498);
    assert_eq!(*heap_value_1, 42);
    assert_eq!(*heap_value_2, 498);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}

use kvos::common::hash::{Mix13Hash,mix13};
use kvos::common::locktable::{LockTable, TryLockResult};
use kvos::common::map::{Map, SimpleHashMap};
use kvos::common::set::SimpleSet;
use kvos::cc::redo::RedoLog;
use kvos::kvstore::{TxKVStore, KVStore};
use alloc::string::String;

#[test_case]
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

#[test_case]
fn test_lock_table_initialization() {
    let table = LockTable::new(10);
    assert_eq!(table.size(), 10, "LockTable size did not match expected value.");
}

#[test_case]
fn test_try_lock_success() {
    let table = LockTable::new(10);
    let result = table.try_lock(5, 1);
    assert_eq!(result, TryLockResult::Success, "Expected to acquire the lock successfully.");
}

#[test_case]
fn test_try_lock_die() {
    let table = LockTable::new(10);
    table.try_lock(5, 1);
    let result = table.try_lock(5, 0);
    assert_eq!(result, TryLockResult::Wait, "Expected to be Wait the lock due to older transaction ID.");
}

#[test_case]
fn test_try_lock_wait() {
    let table = LockTable::new(10);
    table.try_lock(5, 1);
    let result = table.try_lock(5, 2);
    assert_eq!(result, TryLockResult::Die, "Expected to Die for the lock due to younger transaction ID.");
}

#[test_case]
fn test_unlock() {
    let table = LockTable::new(10);
    table.try_lock(5, 1);
    table.unlock(5);
    let result = table.try_lock(5, 2);
    assert_eq!(result, TryLockResult::Success, "Expected to acquire the lock successfully after unlocking.");
}

#[test_case]
fn test_simple_map_insert_and_get() {
    let map: SimpleHashMap<&str, i32> = SimpleHashMap::new();

    // Test insert
    assert!(map.insert(&"key1", &10));
    assert_eq!(map.get(&"key1"), Some(10));

    // Test update
    assert!(map.insert(&"key1", &20));
    assert_eq!(map.get(&"key1"), Some(20));
}

#[test_case]
fn test_simple_map_remove() {
    let map: SimpleHashMap<&str, i32> = SimpleHashMap::new();
    map.insert(&"key1", &10);

    // Test remove existing key
    assert!(map.remove(&"key1"));
    assert_eq!(map.get(&"key1"), None);

    // Test remove non-existing key
    assert!(!map.remove(&"key1"));
}

#[test_case]
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

#[test_case]
fn test_set_insert() {
    let mut set = SimpleSet::<u32>::new();
    assert!(set.insert(&5));
    assert!(!set.insert(&5)); // Duplicate insert should return false
}

#[test_case]
fn test_set_remove() {
    let mut set = SimpleSet::new();
    set.insert(&5);
    assert_eq!(set.remove(&5), true);  // Removing an existing item should return true.
    assert_eq!(set.remove(&5), false); // Removing a non-existent item should return false.
}


#[test_case]
fn test_set_contains() {
    let mut set = SimpleSet::new();
    set.insert(&5);
    assert_eq!(set.contains(&5), true);  // SimpleSet contains the item.
    assert_eq!(set.contains(&10), false); // SimpleSet doesn't contain the item.
}

#[test_case]
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

#[test_case]
fn test_redo_insert() {
    let mut log = RedoLog::new();
    log.insert(&"key1", &"value1");
    assert_eq!(log.get(&"key1"), Some("value1"));

    // Test overwriting existing key
    log.insert(&"key1", &"value2");
    assert_eq!(log.get(&"key1"), Some("value2"));
}

#[test_case]
fn test_redo_update() {
    let mut log = RedoLog::new();
    log.insert(&"key1", &"value1");
    log.update(&"key1", &"updated_value1");
    assert_eq!(log.get(&"key1"), Some("updated_value1"));

    // Test updating non-existent key (shouldn't change anything)
    log.update(&"key2", &"value2");
    assert_eq!(log.get(&"key2"), None);
}

#[test_case]
fn test_redo_remove() {
    let mut log = RedoLog::new();
    log.insert(&"key1", &"value1");
    log.remove(&"key1");
    assert_eq!(log.get(&"key1"), None);
}

#[test_case]
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

#[test_case]
fn test_tx() {
    let kv = TxKVStore::new(32, 32);
    // let tx = kv.begin();
    kv.transact(|tx| {
        tx.write(&String::from("bootloader_name"), &String::from("kvos_boot"));
        let name = tx.read(&String::from("bootloader_name"));
        assert_eq!(name, Some(String::from("kvos_boot")));
    });

    kv.transact(|tx| {
        let name = tx.read(&String::from("bootloader_name"));
        assert_eq!(name, Some(String::from("kvos_boot")));
    });
}
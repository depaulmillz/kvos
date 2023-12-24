use alloc::string::String;
use crate::kvstore::KVStore;
use crate::cc::Transaction;
use crate::KVSTORE;
use crate::console;

pub fn print(s: &str) -> usize {
    print!("{}", s);
    0 
}

pub fn read_kv(keys: &[String], values: &mut [[u8; 32]], len: usize) -> usize {
    KVSTORE.transact_mut(&mut |tx| {
        for i in 0..len {
            // for (j, c) in tx.read(&keys[i]).unwrap().as_bytes().iter().enumerate() {
            //     values[i][j] = *c;
            // }
            let val = tx.read(&keys[i]);
            match val {
                Some(v) => {
                    for (j, c) in v.as_bytes().iter().enumerate() {
                        values[i][j] = *c;
                    }
                },
                None => {
                    for (j, c) in "None".as_bytes().iter().enumerate() {
                        values[i][j] = *c;
                    }
                }
            }
        }
    }, false);
    0
}

pub fn write_kv(keys: &[String], values: &[String], len: usize) -> usize {
    KVSTORE.transact_mut(&mut |tx| {
        for i in 0..len {
            tx.write(&keys[i], &values[i]);
        }
    }, false);
    0
}

pub fn write_kv_persist(keys: &[String], values: &[String], len: usize) -> usize {
    KVSTORE.transact_mut(&mut |tx| {
        for i in 0..len {
            tx.write(&keys[i], &values[i]);
        }
    }, true);
    0
}

pub fn delete_kv(keys: &[String], len: usize) -> usize {
    KVSTORE.transact_mut(&mut |tx| {
        for i in 0..len {
            tx.delete(&keys[i]);
        }
    }, false);
    0
}

pub fn read_in(s: &mut [u8], len: usize) -> usize {
    let stdin = console::read_line(); 
    for (i, c) in stdin.as_bytes().iter().enumerate() {
        if i > len {
            break;
        }
        s[i] = *c;
    }
    stdin.len()
}
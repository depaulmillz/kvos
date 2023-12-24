extern crate alloc;
extern crate spin;
use alloc::vec::Vec;
use spin::Mutex;
struct LockValue {
    locked: bool,
    version_number: u64,
}

#[derive(PartialEq, Debug)]
pub enum TryLockResult {
    Success,
    Wait,
    Die,
}

pub struct LockTable {
    locks: Vec<Mutex<LockValue>>,
    size: u64,
}

impl LockTable {
    pub fn new(size: u64) -> Self {
        let mut locks = Vec::with_capacity(size as usize);
        for _ in 0..size {
            locks.push(Mutex::new(LockValue {
                locked: false,
                version_number: 0,
            }));
        }
        Self { locks:locks, size:size }
    }

    pub fn try_lock(&self, key: u64, transaction_id: u64) -> TryLockResult {
        let mut lock_version = self.locks[key as usize].lock();

        if !lock_version.locked {
            lock_version.locked = true;
            lock_version.version_number = transaction_id;
            return TryLockResult::Success;
        } else if transaction_id > lock_version.version_number {
            return TryLockResult::Die;
        } else {
            return TryLockResult::Wait;
        }
    }

    pub fn unlock(&self, key: u64) {
        let mut lock_value = self.locks[key as usize].lock();
        lock_value.locked = false;
        lock_value.version_number = 0;
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}


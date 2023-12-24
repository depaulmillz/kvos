//!
//! Kernel heap allocator
//!
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
//use crate::memory::allocator::linked_list::LinkedListAlloc;

//use linked_list_allocator::LockedHeap;

//pub mod bump;
//use bump::BumpAllocator;

pub mod linked_list;
//use linked_list::LinkedListAllocator;

pub mod fixed_size_block;
use fixed_size_block::FixedSizeBlockAllocator;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 4096;

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout : Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("Should never dealloc");
    }
}

//#[global_allocator]
//static ALLOCATOR: LockedHeap = LockedHeap::empty();

//#[global_allocator]
//static ALLOCATOR: Locked<BumpAllocator> = Locked::<BumpAllocator>::new(BumpAllocator::new());

//#[global_allocator]
//static ALLOCATOR: Locked<LinkedListAllocator> = Locked::<LinkedListAllocator>::new(LinkedListAllocator::new());

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::<FixedSizeBlockAllocator>::new(FixedSizeBlockAllocator::new());

pub fn init_heap() {
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
}

pub struct Locked<T> {
    inner: spin::Mutex<T>
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}

pub fn align_up(ptr : usize, align: usize) -> usize {
    let remainder = ptr % align;
    if remainder == 0 {
        ptr
    } else {
        ptr + align - remainder
    }
}


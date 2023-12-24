use alloc::{alloc::{Allocator, Layout}, sync::Arc};
use core::{ptr::NonNull, alloc::AllocError};
use spin::Mutex;
use linked_list_allocator::Heap;


pub struct LinkedListAlloc {
    inner: Arc<Mutex<Heap>>
}

impl LinkedListAlloc {
    pub fn new(heap_start: usize, heap_size: usize) -> Self {
        let heap = unsafe { Heap::new(heap_start, heap_size)};
        Self {
            inner: Arc::new(Mutex::new(heap))
        }
    }
}

unsafe impl Allocator for LinkedListAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut allocator = self.inner.lock();
        let res = allocator.allocate_first_fit(layout);
        match res {
            Ok(ptr) => Ok(NonNull::slice_from_raw_parts(ptr, layout.size())),
            Err(_) => Err(AllocError)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let mut allocator = self.inner.lock();
        allocator.deallocate(ptr, layout);
    }
}

impl Clone for LinkedListAlloc {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }
}

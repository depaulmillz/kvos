use super::KernelTest;
use crate::serial_print;
use crate::kstd::{Box, Vec};
use crate::memory::allocator::HEAP_SIZE;

/////////////////////////////////////////////////////////////
//// Tests
////////////////////////////////////////////////////////////

fn simple_allocation() {
    let heap_value_1 = Box::new(42);
    let heap_value_2 = Box::new(498);
    assert_eq!(*heap_value_1, 42);
    assert_eq!(*heap_value_2, 498);
}

fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}

pub fn run_tests() {
    let tests = [
        KernelTest {
            name : "simple_allocation",
            test_fn : simple_allocation,
        },
        KernelTest {
            name : "large_vec",
            test_fn : large_vec,
        },
        KernelTest {
            name : "many_boxes",
            test_fn : many_boxes,
        },
        KernelTest {
            name : "many_boxes_long_lived",
            test_fn : many_boxes_long_lived,
        },
    ];
    for t in tests.iter() {
        serial_print!("{}...\t", t.name);
        (t.test_fn)();
        serial_print!("[ok]\n");
    }
}

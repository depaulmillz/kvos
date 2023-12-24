//!
//! Test framework for testing the OS
//!
mod vga_tests;
mod heap_allocation;
mod kvstore;
mod file_system;
use crate::serial_println;
use crate::serial_print;

//pub trait Testable {
//    fn run(&self) -> ();
//}
//
//impl Testable for fn()
//{
//    fn run(&self) {
//        serial_print!("{}...\t", core::any::type_name::<T>());
//        self();
//        serial_println!("[ok]");
//    }
//}

#[derive(Debug)]
pub struct KernelTest {
    pub name : &'static str,
    pub test_fn : fn(),
}

impl KernelTest {
    pub const fn new(name : &'static str, function : fn()) -> Self {
        Self {
            name : name,
            test_fn : function,
        }
    }
}


fn test_fn() {
    println!("test");
}

pub fn run_tests() {

    let tests = [KernelTest::new("test_fn", test_fn)];

    serial_println!("In run tests");
    for t in tests.iter() {
        serial_print!("{}...\t", t.name);
        (t.test_fn)();
        serial_print!("[ok]\n");
    }
    vga_tests::run_tests();
    heap_allocation::run_tests();
    kvstore::run_tests();
    file_system::run_tests();
    serial_println!("Success");
}


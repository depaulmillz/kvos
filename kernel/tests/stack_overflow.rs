#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kvos::{serial_print};

#[no_mangle]
pub extern "C" fn _start() -> ! {

    serial_print!("stack_overflow::stack_overflow...\t");

    kvos::gdt::init();
    kvos::interrupts::init_test_idt();

    recursive();

    panic!("Should have stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kvos::test_panic_handler(info) 
}

#[allow(unconditional_recursion)]
fn recursive() {
    recursive();
    volatile::Volatile::new(0).read(); // prevent tail recursion optimization
}


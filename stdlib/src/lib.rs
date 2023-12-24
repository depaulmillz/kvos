#![no_std]

extern crate alloc;
extern crate asm;

pub mod syscall;
pub mod shell;
pub mod benchmark;
pub mod rand;

use core::panic::PanicInfo;
use core::arch::asm;
use core::fmt;
use linked_list_allocator::LockedHeap;
pub use alloc::string::{ToString, String};

#[panic_handler]
pub fn panic(_info : &PanicInfo) -> ! {
    loop {}
}

pub fn print(ptr: *const u8, len : usize) {
    unsafe {
        asm!("int 0x80",
             in("rax") 0,
             in("rdi") ptr,
             in("rsi") len);
    }
}

pub fn print_str(s : &str) {
    print(s.as_ptr(), s.len());
}


const HEAP_START : usize = 0x10000000;
const HEAP_SIZE : usize = 4096 * 4;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
}

/// Print a formatted string
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

/// Print a formatted string on a line
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args : fmt::Arguments) {
    let s = args.to_string();
    print(s.as_ptr(), s.len());
}
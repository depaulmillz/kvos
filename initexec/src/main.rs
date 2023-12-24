#![no_std]
#![no_main]

extern crate stdlib;
extern crate alloc;

//use alloc::vec::Vec;
use stdlib::{print_str, ToString};
use stdlib::shell::shell;
//use stdlib::benchmark::run_bench;
#[no_mangle]
extern "C" fn _start() {
    stdlib::init_heap();
    main();
    loop {};
}

extern fn main() {
    print_str("In userspace\n");
    let s = "In userspace with heap\n".to_string();
    stdlib::syscall::print(s.as_ptr(), s.len());
    // let mut keys: Vec<String> = Vec::with_capacity(2);
    // keys.insert(0, "test".to_string());
    // keys.insert(1, "kvos".to_string());

    // let mut values: Vec<String> = Vec::with_capacity(2);
    // values.insert(0, "value should be test".to_string());
    // values.insert(1, "works!".to_string());

    // stdlib::syscall::write_kv(keys.clone(), values.clone());
    // let res = stdlib::syscall::read_kv(keys.clone());
    // println!("{}: {}", keys.get(0).unwrap(), res.get(0).unwrap());
    // println!("{}: {}", keys.get(1).unwrap(), res.get(1).unwrap());
    // run_bench();
    shell();
    loop{}
}


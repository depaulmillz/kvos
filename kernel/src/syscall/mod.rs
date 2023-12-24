use alloc::{slice, string::String};

pub mod numbers;
pub mod funcs;

pub fn dispatcher(n: usize, arg1: usize, arg2: usize, arg3: usize, _arg4: usize) -> usize {
    match n {
        numbers::PRINT => {
            let s = unsafe { core::slice::from_raw_parts(arg1 as *const u8, arg2 as usize) };
            let s = core::str::from_utf8(s).unwrap();
            funcs::print(s);
            0
        },
        numbers::READ_KV => {
            let keys: &[String]  = unsafe { slice::from_raw_parts(arg1 as *mut String, arg3) };
            let values: &mut [[u8; 32]] = unsafe { slice::from_raw_parts_mut(arg2 as *mut [u8; 32], arg3) };        
            funcs::read_kv(keys, values, arg3);
            0
        },
        numbers::WRITE_KV => {
            let keys: &[String]  = unsafe { slice::from_raw_parts(arg1 as *mut String, arg3) };
            let values: &[String] = unsafe { slice::from_raw_parts(arg2 as *mut String, arg3) };        
            funcs::write_kv(keys, values, arg3);
            0
        },
        numbers::DELETE_KV => {
            let keys: &[String]  = unsafe { slice::from_raw_parts(arg1 as *mut String, arg3) };
            funcs::delete_kv(keys, arg3);
            0
        },
        numbers::READ_IN => {
            let s = unsafe { core::slice::from_raw_parts_mut(arg1 as *mut u8, arg2 as usize) };
            funcs::read_in(s, arg2)
        },
        numbers::WRITE_KV_PERSIST => {
            let keys: &[String]  = unsafe { slice::from_raw_parts(arg1 as *mut String, arg3) };
            let values: &[String] = unsafe { slice::from_raw_parts(arg2 as *mut String, arg3) };        
            funcs::write_kv_persist(keys, values, arg3);
            0
        },
        _ => {
            println!("Unknown syscall number: {}", n);
            0
        }
    }
}

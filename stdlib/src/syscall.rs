use core::arch::asm;
use alloc::{vec::Vec, string::String};

pub fn print(ptr: *const u8, len :usize) {
    unsafe{syscall2(0, ptr as usize, len)};
 }
 
pub fn read_kv(keys: Vec<String>) -> Vec<String> {
    // make key ptr
    let len = keys.len();
    let keys_ptr = keys.as_ptr() as usize;
    
    // make val ptr
    let mut vals = Vec::<[u8; 32]>::with_capacity(len);
    for _ in 0..len {
        vals.insert(0, ['\0' as u8; 32]);
    }
    let vals_ptr = vals.as_mut_ptr() as usize;
    
    // cal syscall 
    unsafe { syscall3(1, keys_ptr as usize, vals_ptr, len) };

    // convert vals to Vec<String>
    let mut res = Vec::<String>::with_capacity(len);
    for i in 0..len {
        res.push(String::from_utf8(vals.get(i).unwrap().to_vec()).unwrap());
        res[i] = String::from(res[i].trim_matches(char::from(0)));
    }
    res
}

pub fn write_kv(keys: Vec<String>, values: Vec<String>) {
    // make key ptr
    let len = keys.len();
    let keys_ptr = keys.as_ptr() as usize;
    
    // make val ptr
    let vals_ptr = values.as_ptr() as usize;
    
    // cal syscall 
    unsafe { syscall3(2, keys_ptr as usize, vals_ptr, len) };
}

pub fn delete_kv(keys: Vec<String>) {
    // make key ptr
    let len = keys.len();
    let keys_ptr = keys.as_ptr() as usize;
    
    // cal syscall 
    unsafe { syscall2(3, keys_ptr as usize, len) };
}


pub fn write_kv_persist(keys: Vec<String>, values: Vec<String>) {
    // make key ptr
    let len = keys.len();
    let keys_ptr = keys.as_ptr() as usize;
    
    // make val ptr
    let vals_ptr = values.as_ptr() as usize;
    
    // cal syscall 
    unsafe { syscall3(5, keys_ptr as usize, vals_ptr, len) };
}


pub fn read_in(s: &mut [u8], len: usize) -> usize {
    unsafe { syscall2(4, s.as_mut_ptr() as usize, len) }
}


pub unsafe fn syscall0(n: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        lateout("rax") res
    );
    res
}

pub unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1,
        lateout("rax") res
    );
    res
}

pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2,
        lateout("rax") res
    );
    res
}

pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2, in("rdx") arg3,
        lateout("rax") res
    );
    res
}

pub unsafe fn syscall4(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2, in("rdx") arg3, in("r8") arg4,
        lateout("rax") res
    );
    res
}

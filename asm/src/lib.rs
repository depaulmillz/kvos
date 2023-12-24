#![no_std]

extern crate alloc;

use core::arch::asm;
use alloc::string::String;
use core::str::from_utf8;
use core::arch::x86_64;

/// Convert u32 to a byte aray for parsing
fn u32_to_byte_array(x : u32) -> [u8; 4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    [b4, b3, b2, b1]
}

/// Take cpuid result and parse to a string
fn to_string(res : x86_64::CpuidResult) -> String {
    let mut data = [0u8;12];
    let ebx_a = u32_to_byte_array(res.ebx);
    let edx_a = u32_to_byte_array(res.edx);
    let ecx_a = u32_to_byte_array(res.ecx);
    data[0..4].clone_from_slice(&ebx_a);
    data[4..8].clone_from_slice(&edx_a);
    data[8..12].clone_from_slice(&ecx_a);
    String::from(from_utf8(&data).expect("Should contain results"))
}

/// CPUID infomation
#[derive(Debug)]
pub struct CPUID {
    /// Processor name
    pub processor_name : String,
    /// Local apic id
    pub local_apic_id : u32,
    /// If we have TSC
    pub tsc : bool,
    /// If we have APIC
    pub apic : bool,
    /// If we have sysenter and exit support
    pub sysenter_and_exit : bool,
    /// If we have htt support
    pub htt : bool,
    /// If we have tsc deadline support
    pub tsc_deadline : bool,
    /// If we have RDTSCP support
    pub rdtscp : bool,
    /// If we have TSC invariant
    pub tsc_invariant : bool
}

impl CPUID {
    /// Call CPU id and get information
    pub fn new() -> CPUID {
       
        let processor_name = to_string( unsafe { x86_64::__cpuid(0) } );
        
        let h1 = unsafe { x86_64::__cpuid(1) };

        let local_apic_id = (h1.edx >> 24) & 0x7F;
        
        let tsc = ((h1.edx >> 4) & 0x1) == 0x1;

        let apic = ((h1.edx >> 9) & 0x1) == 0x1;
        
        let sysenter_and_exit = ((h1.edx >> 11) & 0x1) == 0x1;
        
        let htt = ((h1.edx >> 28) & 0x1) == 0x1;
        
        let tsc_deadline = ((h1.ecx >> 24) & 0x1) == 0x1;

        let h80000001 = unsafe { x86_64::__cpuid(0x80000001) };

        let rdtscp = ((h80000001.edx >> 27) & 0x1) == 0x1;

        let h80000007 = unsafe { x86_64::__cpuid(0x80000007) };

        let tsc_invariant = ((h80000007.edx >> 8) & 0x1) == 0x1;

        CPUID {
            processor_name,
            local_apic_id,
            tsc,
            apic,
            sysenter_and_exit,
            htt,
            tsc_deadline,
            rdtscp,
            tsc_invariant,
        }
    }
}

/// Read timestamp
pub fn rdtsc() -> u64 {
    unsafe { x86_64::_rdtsc() }
}

/// LFence instruction
pub fn lfence() {
    unsafe {
        asm!("lfence");
    }
}

/// MFence instruction
pub fn mfence() {
    unsafe {
        asm!("mfence");
    }
}

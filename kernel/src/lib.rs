// test lib
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(new_uninit)]
#![feature(type_name_of_val)]
#![feature(naked_functions)]
#![feature(allocator_api)]

use core::panic::PanicInfo;

extern crate bitflags;
extern crate object;
extern crate alloc;
extern crate multiboot2;
extern crate asm;

#[macro_use]
pub mod drivers;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod map;
pub mod kstd;
pub mod cc;
pub mod disk;
pub mod kvstore;
pub mod common {
    pub mod hash;
    pub mod set;
    pub mod map;
    pub mod locktable;
}
pub mod tests;
pub mod apic;
pub mod userspace;
pub mod syscall;
pub mod acpi;
pub mod console;
pub mod benchmark;

use alloc::{sync::Arc, string::String};
use multiboot2::BootInformation;
use lazy_static::lazy_static;
use crate::kvstore::{KVStore, TxKVStorePersist};

//pub mod task

/// Set if testing
static TESTING : core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

lazy_static! {
    pub static ref KVSTORE: Arc<TxKVStorePersist<String, String>>  = TxKVStorePersist::new(1024, 1024);
}


#[panic_handler]
fn panic(info : &PanicInfo) -> ! {
    // If we are testing use serial println
    if TESTING.load(core::sync::atomic::Ordering::Relaxed) {
        serial_println!("[failed]");
        serial_println!("Error: {}", info);
        exit_qemu(QemuExitCode::Failed);
        hlt_loop()
    } else {
        // If we are not testing print and halt
        println!("{}", info);
        hlt_loop()
    }
}

/// QemuExit code to write over serial port
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    /// Sucessfully exited
    Success = 0x10,
    /// Failed
    Failed = 0x11,
}

/// Exit qemu
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// Enable no execute pages
fn enable_nxe_bit() {
    use x86_64::registers::model_specific::{EferFlags, Efer};

    let mut efer = Efer::read();

    efer |= EferFlags::NO_EXECUTE_ENABLE;

    unsafe {
        Efer::write(efer);
    }
}

/// Enable write protection for unwritable pages
fn enable_write_protect_bit() {
    use x86_64::registers::control::{Cr0, Cr0Flags};
    let mut flags = Cr0::read();
    flags |= Cr0Flags::WRITE_PROTECT;
    unsafe {
        Cr0::write(flags);
    }
}

/// Initialize kernel
pub fn init(kernel_start : usize, kernel_end : usize, multiboot_start: usize, multiboot_end : usize, boot_info : &BootInformation) -> memory::paging::frameallocator::AreaFrameAllocator<'_> {
    serial_infoln!("Enable nxe bit"); 
    enable_nxe_bit();
    serial_infoln!("Enable write protection bit"); 
    enable_write_protect_bit();

    let rsdpv1 = boot_info.rsdp_v1_tag().expect("ACPI >=v2 no implemented");
    if !rsdpv1.checksum_is_valid() {
        panic!("Invalid checksum");
    }
    let rsdt_addr : *const u8 = rsdpv1.rsdt_address() as *const u8;

    let (ioapic_info, cpus) = acpi::init(rsdt_addr).expect("No ioapic");

    drivers::timing::init();

    serial_debugln!("Init gdt");
    gdt::init();
    serial_debugln!("Init idt");
    interrupts::init_idt();


    let mut frame_allocator = memory::paging::frameallocator::AreaFrameAllocator::new(
            kernel_start as usize, kernel_end as usize, multiboot_start,
            multiboot_end, &boot_info);
    serial_debugln!("Disable pic, enable apic");
    interrupts::init_pic(ioapic_info, &mut frame_allocator);

    interrupts::init_interrupts();
    println!("Init interrupts");

    for idx in 0..cpus.size() {
        let info = cpus.get(idx).expect("should be able to get this cpu");
        if  info.apic_id as usize != interrupts::LOCAL_APIC.get_apic_id() {
            println!("Have core {} as well", info.apic_id);
        }
    }
   
    println!("Remapping the kernel");
    memory::paging::remap_the_kernel(&mut frame_allocator, &boot_info);

    println!("Init heap");
    memory::allocator::init_heap();

    drivers::timing::init_nanosleep();

    frame_allocator
}

/// Breakpoint
pub fn breakpoint() {
    x86_64::instructions::interrupts::int3();
}

/// Halt
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Kernel main function
#[no_mangle]
pub extern "C" fn kernel_main(multiboot_information_address: usize, stack_start : usize, stack_end : usize) -> ! {

    serial_traceln!("KVOS");

    clear_vga!();
    println!("KVOS");

    serial_infoln!("Stack: 0x{:x} to 0x{:x} will be identity mapped", stack_start, stack_end);
    
    let boot_info = unsafe { multiboot2::load(multiboot_information_address).unwrap() };



    let memory_map_tag : &multiboot2::MemoryMapTag = boot_info.memory_map_tag().expect("Require memory map tag");

    for area in memory_map_tag.memory_areas() {
        serial_infoln!("Memory area: start 0x{:x} length {:x}", area.start_address(), area.size());
    }

    let elf_sections_tag = boot_info.elf_sections_tag().expect("Require elf sections tag");

    let kernel_start = elf_sections_tag.sections().map(|s| s.start_address()).min().unwrap() as usize;
    let kernel_end = elf_sections_tag.sections().map(|s| s.end_address()).max().unwrap() as usize;

    serial_traceln!("Kernel start 0x{:x} and end 0x{:x} will be identity mapped", kernel_start, kernel_end);
   
    let multiboot_start = multiboot_information_address;
    let multiboot_end = multiboot_start + (boot_info.total_size() as usize);
    serial_traceln!("Multiboot start 0x{:x} and end 0x{:x} will be identity mapped", multiboot_start, multiboot_end);

    let bootloader_name = boot_info.boot_loader_name_tag().expect("Expect bootloader name").name();
    serial_infoln!("Boot loader name tag: {}", bootloader_name);
 
    
    let mut frame_allocator = init(kernel_start, kernel_end, multiboot_start, multiboot_end, &boot_info);

    let cpuid = asm::CPUID::new();
    serial_infoln!("CPU Info {:?}", cpuid);

    serial_traceln!("Current time {}", asm::rdtsc());
    serial_traceln!("Next time {}", asm::rdtsc());   
    serial_traceln!("Next time {}", asm::rdtsc());

    // use crate::cc::Transaction;    
    
    // // These lines have to be commented out to compile.
    // KVSTORE.transact(|tx| {
    //     tx.write(&String::from("bootloader_name"), &String::from(bootloader_name));
    // });

    // KVSTORE.transact(|tx| {
    //     tx.write(&String::from("os_name"), &String::from("KVOS"));
    //     tx.write(&String::from("stack_start"), &stack_start.to_string());
    //     tx.write(&String::from("stack_end"), &stack_end.to_string());
    //     tx.write(&String::from("kernel_start"), &kernel_start.to_string());
    //     tx.write(&String::from("kernel_end"), &kernel_end.to_string());
    //     tx.write(&String::from("multiboot_start"), &multiboot_start.to_string());
    //     tx.write(&String::from("multiboot_end"), &multiboot_end.to_string());
    //     tx.write(&String::from("multiboot_information_address"), &multiboot_information_address.to_string());
    // });

    // {

    //     let mut bootloader_name : String = "".to_string();
    //     let mut stack_start : String = "".to_string();
    //     let mut stack_end : String = "".to_string();


    //     KVSTORE.transact_mut(&mut |tx| {
    //             bootloader_name = tx.read(&String::from("bootloader_name")).unwrap(); // Panicking here
    //             stack_start = tx.read(&String::from("stack_start")).unwrap();
    //             stack_end = tx.read(&String::from("stack_end")).unwrap();
    //     });
    
    //     println!("Bootloader name is {}", bootloader_name);
    // }
    // benchmark::run_bench();
    // Test vs Bench vs Run Userspace
    
    let command_line = { 
        match boot_info.command_line_tag() {
            Some(cl) => cl.command_line(),
            _ => ""
        }
    };

    //drivers::buzzer::songs(); //buzz(329.63, 100.0);    

    if command_line == "test" {
        TESTING.store(true, core::sync::atomic::Ordering::SeqCst);
        println!("testing...");
        tests::run_tests(); 
        println!("Testing done");
        exit_qemu(QemuExitCode::Success);
        hlt_loop();
    } else if command_line == "bench" {
        println!("benchmarking...");
        hlt_loop();
    } else {
        println!("Got command line {}", command_line);
    }
   
    userspace::initproc(&boot_info, &mut frame_allocator);

}

//!
//! Create interrupts
//!

pub mod apic;

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{println, gdt, print, hlt_loop, syscall};
use crate::drivers::timing;
use crate::acpi::IOAPICInfo;
use lazy_static::lazy_static;
use core::arch::asm;
use spin;
use crate::apic::{LAPIC, IOAPIC};
use crate::memory::paging::frameallocator::FrameAllocator;
use crate::console::key_handle;

//// Global variables

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        unsafe {
            idt.general_protection_fault.set_handler_fn(general_protection_fault_handler)
                .set_stack_index(gdt::GENERAL_PROTECTION_FAULT_IST_INDEX);

            idt.page_fault.set_handler_fn(page_fault_handler)
                .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
            idt[0x80]
                .set_handler_fn(core::mem::transmute(syscall_handler as *mut fn()))
                .set_privilege_level(x86_64::PrivilegeLevel::Ring3);
        }

        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.cp_protection_exception.set_handler_fn(cp_protection_handler);
        idt.hv_injection_exception.set_handler_fn(hv_injection_handler);
        idt.vmm_communication_exception.set_handler_fn(vmm_communication_handler);
        idt.security_exception.set_handler_fn(security_exception_handler);
       


        idt[32]
            .set_handler_fn(timer_interrupt_handler);
        idt[33]
            .set_handler_fn(keyboard_interrupt_handler);
        idt[39]
            .set_handler_fn(spurious_interrupt_handler);
        //idt[InterruptIndex::Keyboard.as_usize()]
        //    .set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

//// Functions

/// Initialize the idt
pub fn init_idt() {
    IDT.load();
}

pub static LOCAL_APIC : LAPIC = LAPIC::zeroed();
pub static IOAPIC : IOAPIC = IOAPIC::zeroed();

pub fn init_pic<A>(ioapic_info : IOAPICInfo, alloc : &mut A) 
where A : FrameAllocator
{
    // disable pic
    use x86_64::instructions::port::Port;
    let mut ha1port = Port::<u8>::new(0xa1);
    unsafe { ha1port.write(0xff) };

    let mut h21port = Port::<u8>::new(0x21);
    unsafe { h21port.write(0xff) };

    // setup local_apic
    LOCAL_APIC.init(alloc);

    // setup ioapic
    IOAPIC.init(ioapic_info, LOCAL_APIC.get_apic_id() as u32, alloc);
}

/// Enable interrupts
pub fn init_interrupts() {
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame,
                                               _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE_FAULT\n{:#?}", stack_frame)
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame,
                                             error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;
    println!("EXCEPTION: PAGE_FAULT\n{:#?}", stack_frame);
    println!("Accessed Addr {:?}", Cr2::read());
    println!("Error code {:?}", error_code);
    hlt_loop();
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW \n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, _error_code : u64) {
    println!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, _error_code : u64) {
    println!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, _error_code : u64) {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, _error_code : u64) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, _error_code : u64) {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop()
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn cp_protection_handler(stack_frame: InterruptStackFrame, _error_code : u64) {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn hv_injection_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn vmm_communication_handler(stack_frame: InterruptStackFrame, _error_code : u64) {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn security_exception_handler(stack_frame: InterruptStackFrame, _error_code : u64) {
    println!("EXCEPTION: \n{:#?}", stack_frame);
    hlt_loop();
}

// PIC

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame) {

    timing::add_tick();
    LOCAL_APIC.eoi();
 
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame) {

    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                                     HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60); // PS/2
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => key_handle(character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    LOCAL_APIC.eoi();
}

extern "x86-interrupt" fn spurious_interrupt_handler(
    stack_frame: InterruptStackFrame) {

    println!("SPURIOUS INTERRUPT: \n{:?}", stack_frame);
    LOCAL_APIC.eoi();
  
    //unsafe {
    //    PICS.lock()
    //        .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    //}
}

//extern "x86-interrupt" fn keyboard_interrupt_handler(
//    _stack_frame: InterruptStackFrame) {
//    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
//    use spin::Mutex;
//    use x86_64::instructions::port::Port;
//
//    lazy_static! {
//        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
//            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
//                                     HandleControl::Ignore));
//    }
//
//    let mut keyboard = KEYBOARD.lock();
//    let mut port = Port::new(0x60); // PS/2
//    let scancode: u8 = unsafe { port.read() };
//    //crate::task::keyboard::add_scancode(scancode); from task
//
//
//    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
//        if let Some(key) = keyboard.process_keyevent(key_event) {
//            match key {
//                DecodedKey::Unicode(character) => print!("{}", character),
//                DecodedKey::RawKey(key) => print!("{:?}", key),
//            }
//        }
//    }
//
//    unsafe {
//        PICS.lock()
//            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
//    }
//}

//// TESTING

#[naked]
extern "sysv64" fn syscall_handler() {
    unsafe {
        asm!(
            "push rax",
            "push rcx",
            "push rdx",
            "push rsi",
            "push rdi",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "mov rsi, rsp", // Arg #2: register list
            "mov rdi, rsp", // Arg #1: interupt frame
            "add rdi, 9 * 8", // stack ptr + everything that is pushed
            "call {}",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop rax",
            "iretq",
            sym syscall_handler_impl,
            options(noreturn)
        );
    }
}

#[allow(dead_code)]
struct Registers {
    r11 : u64,
    r10 : u64,
    r9 : u64,
    r8 : u64,
    rdi : u64,
    rsi : u64,
    rdx : u64,
    rcx : u64,
    rax : u64,
}

extern "sysv64" fn syscall_handler_impl(_stack_frame: &mut InterruptStackFrame, regs : &mut Registers) {
    // get arguments
    let n    = regs.rax as usize;
    let arg1 = regs.rdi as usize;
    let arg2 = regs.rsi as usize;
    let arg3 = regs.rdx as usize;
    let arg4 = regs.r8 as usize;


    let res = syscall::dispatcher(n, arg1, arg2, arg3, arg4);

    regs.rax = res as u64;

    LOCAL_APIC.eoi();
}


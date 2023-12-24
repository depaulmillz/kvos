//!
//! GDT code
//!
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;
pub const GENERAL_PROTECTION_FAULT_IST_INDEX: u16 = 2;

/// GDT entries
pub struct Selectors {
    /// Kernel code entry
    code_selector: SegmentSelector,
    /// Kernel data entry
    data_selector: SegmentSelector,
    /// TSS entry
    tss_selector: SegmentSelector,
    /// User data entry
    pub user_data_selector: SegmentSelector,
    /// User code entry
    pub user_code_selector: SegmentSelector
}

/// TSS stack size
const STACK_SIZE: usize = 4096 * 5;

lazy_static! {
    /// TSS stack
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Set privledge stack table
        tss.privilege_stack_table[0] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // our safe stack
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };

        // Set interrupt stack table for double fault
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // our safe stack
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };
        
        // Set interrupt stack table for page fault
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // our safe stack
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };

        // Set interrupt stack table for general protection fault
        tss.interrupt_stack_table[GENERAL_PROTECTION_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // our safe stack
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };
        tss
    };
}

lazy_static! {
    /// GDT and entries
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());

        (gdt, Selectors { code_selector, data_selector, tss_selector, user_data_selector, user_code_selector })
    };
}

/// Initialize GDT
pub fn init() {

    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, SS, DS, ES, FS, GS, Segment};

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        SS::set_reg(SegmentSelector::NULL); // can be null
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(SegmentSelector::NULL);
        FS::set_reg(SegmentSelector::NULL); // used in user mode for TLS
        GS::set_reg(SegmentSelector::NULL); // pointer to per cpu kernel data structure
        load_tss(GDT.1.tss_selector);       // load the TSS
    }
}


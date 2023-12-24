//! Paging and memory management of frames
pub mod mapper;
pub mod translation;
pub mod frameallocator;
pub mod entry;
pub mod table;
pub mod temporary_page;

use multiboot2::BootInformation;
use core::ops::{Deref, DerefMut};
use crate::interrupts::LOCAL_APIC;

use super::allocator::{HEAP_START, HEAP_SIZE};
use entry::EntryFlags;

pub type VirtualAddress = usize;
pub const PAGE_SIZE: usize = 4096;
pub type PhysicalAddress = usize;

pub struct ActivePageTable {
    mapper: mapper::Mapper,
}

impl Deref for ActivePageTable {
    type Target = mapper::Mapper;
    fn deref(&self) -> &mapper::Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {

    fn deref_mut(&mut self) -> &mut mapper::Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: mapper::Mapper::new(),
        }
    }

    pub fn with<F>(&mut self,
                   table: &mut InactivePageTable,
                   temporary_page: &mut temporary_page::TemporaryPage,
                   f: F)
        where F : FnOnce(&mut mapper::Mapper)
    {

        use x86_64::instructions::tlb;
        use x86_64::registers::control::Cr3;

        let (phys_frame , _) = Cr3::read();

        let cr3_val = phys_frame.start_address().as_u64() as usize;


        let backup = translation::Frame::containing_address( cr3_val );

        let p4_table = temporary_page.map_table_frame(backup.clone(), self);

        self.p4_mut()[511].set(table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        tlb::flush_all();

        f(self);

        p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
        tlb::flush_all();

        temporary_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {

        assert!(self.translate(new_table.p4_frame.start_address()).is_some());


        use x86_64::registers::control::Cr3;

        let (phys_frame , flags) = Cr3::read();
        let cr3_val = phys_frame.start_address().as_u64() as usize;
        let old_table = InactivePageTable {
            p4_frame : translation::Frame::containing_address(cr3_val)
        };


        let addr = x86_64::addr::PhysAddr::new(new_table.p4_frame.start_address() as u64);

        let p_frame = x86_64::structures::paging::PhysFrame::containing_address(addr);

        unsafe {
            Cr3::write(p_frame, flags);
        }
        old_table
    }

}

pub struct InactivePageTable {
    p4_frame: translation::Frame,
}

impl InactivePageTable {
    pub fn new(frame: translation::Frame,
               active_table: &mut ActivePageTable,
               temporary_page: &mut temporary_page::TemporaryPage) -> InactivePageTable {
        // zero and recursive map the frame
       
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
       
        temporary_page.unmap(active_table);
        InactivePageTable { p4_frame: frame }
    }
}

pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation) 
where A: frameallocator::FrameAllocator
{


    // assume deadbeaf is unused
    let mut tmp_page = temporary_page::TemporaryPage::new(translation::Page { number: 0xdeadbeaf }, allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    //assert!(active_table.translate_page(translation::Page { number : 0xdeadbeaf }) == None);
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut tmp_page)
    };

    active_table.with(&mut new_table, &mut tmp_page, |mapper| {
        use translation::Frame;

        serial_debugln!("Mapping vga");
        mapper.identity_map(Frame::containing_address(0xb8000), EntryFlags::WRITABLE, allocator);
        mapper.identity_map(Frame::containing_address(LOCAL_APIC.get_ptr()), EntryFlags::WRITABLE | EntryFlags::NO_CACHE, allocator);

        serial_debugln!("Mapping multiboot");
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }

        let elf_sections_tag = boot_info.elf_sections_tag().expect("Require elf sections tag");
        
        serial_debugln!("Mapping kernel");
        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                continue;
            }
            assert!((section.start_address() as usize) % PAGE_SIZE == 0, "page alignment required for elf section {}", section.name());
            let flags = EntryFlags::from_elf_section_flags(&section);
            let start_frame = Frame::containing_address(section.start_address() as usize);
            let end_frame = Frame::containing_address(section.end_address() as usize - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        serial_debugln!("Mapping modules");
        for module in boot_info.module_tags() {
            assert!((module.start_address() as usize) % PAGE_SIZE == 0, "page alignment required for module");
            let start_frame = Frame::containing_address(module.start_address() as usize); 
            let end_frame = Frame::containing_address(module.end_address() as usize - 1); 
            serial_debugln!("Module identity mapping from 0x{:x} to 0x{:x} rounded up to 0x{:x}", start_frame.clone().start_address(), module.end_address(), end_frame.clone().start_address() + 4093);
            for frame in Frame::range_inclusive(start_frame.clone(), end_frame.clone()) {
                mapper.try_identity_map(frame, EntryFlags::PRESENT, allocator);
            }
            serial_debugln!("Module identity mapped from 0x{:x} to 0x{:x}", start_frame.clone().start_address(), end_frame.clone().start_address() + 4093);
        }

    });

    let old_table = active_table.switch(new_table);

    let old_p4_page = translation::Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page.clone(), allocator);

    let start_heap = translation::Page::containing_address(HEAP_START);
    let end_heap = translation::Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    serial_debugln!("Heap from 0x{:x} to 0x{:x}", start_heap.clone().start_address(), end_heap.clone().start_address() + 4093);

    for page in translation::Page::range_inclusive(start_heap, end_heap) {
        assert!(active_table.translate(page.start_address()).is_none());
        let flags = EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE;
        active_table.map(page.clone(), flags, allocator);
        assert!(active_table.translate(page.start_address()).is_some());
    }

}


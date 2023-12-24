pub mod paging;
pub mod allocator;

pub use paging::entry::EntryFlags;

pub unsafe fn map_memory<A>(addr : paging::VirtualAddress, length : usize, flags : EntryFlags, alloc : &mut A) -> paging::VirtualAddress 
where A : paging::frameallocator::FrameAllocator
{
    map_memory_impl(addr, length, flags, alloc)
}

pub unsafe fn map_memory_expect<A>(addr : paging::VirtualAddress, length : usize, flags : EntryFlags, alloc : &mut A) -> Result<paging::VirtualAddress, ()>
where A : paging::frameallocator::FrameAllocator
{
    map_memory_expect_impl(addr, length, flags, alloc)
}

fn map_memory_impl<A>(addr : paging::VirtualAddress, length : usize, flags : EntryFlags, alloc : &mut A) -> paging::VirtualAddress 
where A : paging::frameallocator::FrameAllocator
{
    use paging::translation::Page;

    let mut active_page_table = unsafe { crate::memory::paging::ActivePageTable::new() };

    // never map between 0x0 and 0x1000

    if addr != 0x0 && addr > 0x1000 {
        serial_traceln!("Trying to map in at {:x}", addr);
        let start_page = Page::containing_address(addr);
        let end_page = Page::containing_address(addr + length);

        serial_traceln!("Considering pages {:?} to {:?}", start_page, end_page);

        let mut found = false;
        for p in Page::range_inclusive(start_page.clone(), end_page.clone()) {
            if active_page_table.translate_page(p.clone()).is_some() {
                serial_traceln!("Found a page there");
                found = true;
            }
        }
        if !found {
            serial_debugln!("Able to map in requested area at {:x}", addr);
            // we can map it in and allocate
            for p in Page::range_inclusive(start_page, end_page) {
                active_page_table.map(p, flags, alloc);
            }
            return addr;
        }
    }

    let mut start_page = Page::containing_address(0x1000);

    let mut not_found = true;

    while not_found {
        let mut first_found : Option<Page> = None;

        if start_page.start_address().saturating_add(length) == usize::MAX {
            return 0;
        }

        not_found = false;
        for p in Page::range_inclusive(start_page.clone(), Page::containing_address(start_page.start_address() + length)) {
            let found = active_page_table.translate_page(p.clone()).is_some();
            if first_found.is_none() && found {
                first_found = Some(p.clone());  
            }
            if !found {
                not_found = true;
            }
        }

        if not_found {
            if let Some(page) = first_found {
                start_page = page;
            } else {
                let mut p = Page::containing_address(start_page.start_address() + length);
                p.number += 1;
                start_page = p;
            } 
        }
    }
    
    for p in Page::range_inclusive(start_page.clone(), Page::containing_address(start_page.start_address() + length)) {
        serial_debugln!("Mapping in page at {:x} in map_memory", start_page.start_address());
        active_page_table.map(p, flags, alloc);
    }
    start_page.start_address()
}

fn map_memory_expect_impl<A>(addr : paging::VirtualAddress, length : usize, flags : EntryFlags, alloc : &mut A) -> Result<paging::VirtualAddress, ()>
where A : paging::frameallocator::FrameAllocator
{
    use paging::translation::Page;

    let mut active_page_table = unsafe { crate::memory::paging::ActivePageTable::new() };

    // never map between 0x0 and 0x1000

    if addr != 0x0 && addr > 0x1000 {
        serial_traceln!("Trying to map in at {:x}", addr);
        let start_page = Page::containing_address(addr);
        let end_page = Page::containing_address(addr + length);

        serial_traceln!("Considering pages {:?} to {:?}", start_page, end_page);

        let mut found = false;
        for p in Page::range_inclusive(start_page.clone(), end_page.clone()) {
            if active_page_table.translate_page(p.clone()).is_some() {
                serial_traceln!("Found a page there");
                found = true;
            }
        }
        if !found {
            serial_debugln!("Able to map in requested area at {:x}", addr);
            // we can map it in and allocate
            for p in Page::range_inclusive(start_page, end_page) {
                active_page_table.map(p, flags, alloc);
            }
            return Ok(addr);
        }
    }

    Err(())
}

pub unsafe fn change_map_memory(addr : paging::VirtualAddress, length : usize, flags : EntryFlags)
{
    change_map_memory_impl(addr, length, flags)
}

fn change_map_memory_impl(addr : paging::VirtualAddress, length : usize, flags : EntryFlags) {
    use paging::translation::Page;
    let mut active_page_table = unsafe { crate::memory::paging::ActivePageTable::new() };
    
    let start_page = Page::containing_address(addr);
    for p in Page::range_inclusive(start_page.clone(), Page::containing_address(start_page.start_address() + length)) {
        serial_debugln!("Changing flags at {:x} to {:?}", p.start_address(), flags);
        active_page_table.change_flags(p, flags);
    }
}

use core::ptr::Unique;
use super::table::{Table, Level4, P4};
use super::frameallocator::FrameAllocator;
use super::entry::{EntryFlags, ENTRY_COUNT};
use super::{PAGE_SIZE, VirtualAddress, PhysicalAddress};
use super::translation::{Page, Frame};

pub struct Mapper {
    p4: Unique<Table<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: Unique::new_unchecked(P4),
        }
    }

    pub fn change_p4(&mut self, new_p4 : *mut Table<Level4>) {
        unsafe {
            self.p4 = Unique::new_unchecked(new_p4);
        }
    }

    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A) 
    where A : FrameAllocator
    {
        assert!(self.translate(page.start_address()).is_none());

        let p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);
    
        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
        assert!(self.translate(page.start_address()).is_some());
    }
    
    pub fn change_flags(&mut self, page: Page, flags: EntryFlags) 
    {
        assert!(self.translate(page.start_address()).is_some());

        let p3 = self.p4_mut().next_table_mut(page.p4_index()).expect("Cant get next table");
        if p3[page.p3_index()].flags().contains(EntryFlags::HUGE_PAGE) {
            panic!("Unable to change flags");
        }

        let p2 = p3.next_table_mut(page.p3_index()).expect("Cant get next table");

        if p2[page.p2_index()].flags().contains(EntryFlags::HUGE_PAGE) {
            panic!("Unable to change flags");
        }

        let p1 = p2.next_table_mut(page.p2_index()).expect("Cant get next table");
    
        assert!(!p1[page.p1_index()].is_unused());
        let frame = p1[page.p1_index()].pointed_frame().expect("Expect frame to exist");
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
        assert!(self.translate(page.start_address()).is_some());
    }
    
    pub fn translate(&self, addr: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = addr % PAGE_SIZE;
        let optional_frame = self.translate_page(Page::containing_address(addr));
        if let Some(frame) = optional_frame {
            Some(frame.number * PAGE_SIZE + offset) 
        } else {
            None
            
        }
            //.map(|frame| frame.number * PAGE_SIZE + offset)
    }

    pub fn translate_page(&self, page: Page) -> Option<Frame> {

        let p3 = self.p4().next_table(page.p4_index());
    
        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];
                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                        // 1GiB aligned
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some(Frame {
                            number : start_frame.number + page.p2_index() * ENTRY_COUNT + page.p1_index(),
                        });
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MiB page
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                            // 2MiB aligned
                            assert!(start_frame.number % ENTRY_COUNT == 0);
                            return Some(Frame {
                                number : start_frame.number +  page.p1_index(),
                            });
                        }
                    }
                }
                None
            })
        };
    
        p3.and_then(|p3| p3.next_table(page.p3_index()))
          .and_then(|p2| p2.next_table(page.p2_index()))
          .and_then(|p1| p1[page.p1_index()].pointed_frame())
          .or_else(huge_page)
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator 
    {
        if self.translate(page.start_address()).is_some() {
            panic!("Mapping 0x{:x} but found addr maps to 0x{:x}", page.start_address(), self.translate(page.start_address()).unwrap());
        }
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator 
    {
        if !self.translate(frame.start_address()).is_none() {
            panic!("Mapping identity 0x{:x} but found addr maps to 0x{:x}", frame.start_address(), self.translate(frame.start_address()).unwrap());
        }
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    pub fn try_identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A) -> bool
        where A: FrameAllocator 
    {
        if !self.translate(frame.start_address()).is_none() {
            serial_errorln!("Mapping identity 0x{:x} but found addr maps to 0x{:x}", frame.start_address(), self.translate(frame.start_address()).unwrap());
            false
        } else {
            let page = Page::containing_address(frame.start_address());
            self.map_to(page, frame, flags, allocator);
            true
        }
    }

    pub fn unmap<A>(&mut self, page: Page, _allocator : &mut A)
        where A: FrameAllocator
    {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
                     .next_table_mut(page.p4_index())
                     .and_then(|p3| p3.next_table_mut(page.p3_index()))
                     .and_then(|p2| p2.next_table_mut(page.p2_index()))
                     .expect("doesn not support huge pages");
        let _frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();

        use x86_64::instructions::tlb;
        use x86_64::addr;
        tlb::flush(addr::VirtAddr::new(page.start_address() as u64));

        assert!(self.translate(page.start_address()).is_none());
        //allocator.deallocate_frame(frame);
    }

}

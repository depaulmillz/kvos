use super::translation::{Frame, Page};
use super::table::{Table, Level1};
use super::{VirtualAddress, ActivePageTable};
use super::frameallocator::FrameAllocator;

#[derive(Debug)]
pub struct TemporaryPage {
    page : Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {

    pub fn new<A>(page: Page, allocator: &mut A) -> TemporaryPage
    where A: FrameAllocator
    {
        TemporaryPage {
            page: page,
            allocator: TinyAllocator::new(allocator),
        }
    }
    
    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        use super::entry::EntryFlags;
        assert!(active_table.translate_page(self.page.clone()).is_none(), "temporary page is already mapped");
        active_table.map_to(self.page.clone(), frame, EntryFlags::WRITABLE, &mut self.allocator);
        assert!(active_table.translate_page(self.page.clone()).is_some(), "mapping not successful");

        self.page.start_address()
    }

    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page.clone(), &mut self.allocator);
    }

    pub fn map_table_frame(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }
}

#[derive(Debug)]
struct TinyAllocator([Option<Frame>; 3]);

impl FrameAllocator for TinyAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        panic!("Need to free for tiny allocator");
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("Tiny allocator can hold only 3 frames.");
    }
}

impl TinyAllocator {
    fn new<A>(allocator: &mut A) -> TinyAllocator
        where A: FrameAllocator
    {
        let mut f = || allocator.allocate_frame();
        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }
}


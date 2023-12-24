use super::{PAGE_SIZE, VirtualAddress, PhysicalAddress};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    pub number : usize,
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000
               || address >= 0xffff_8000_0000_0000, "invalid addr 0x{:x}",
               address);
        Page { number : address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }
    
    pub fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    pub fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    pub fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    pub fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }

    pub fn clone(&self) -> Page {
        Page { number : self.number }
    }

    pub fn range_inclusive(start_page : Page, end_page : Page) -> PageIter {
        PageIter {
            start : start_page,
            end : end_page,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number : usize
}

impl Frame {
    pub fn containing_address(address: usize) -> Frame {
        Frame { number : address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    pub fn clone(&self) -> Frame {
        Frame { number : self.number }
    }

    pub fn range_inclusive(start_frame : Frame, end_frame : Frame) -> FrameIter {
        FrameIter {
            start : start_frame,
            end : end_frame,
        }
    }
}

pub struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let tmp = self.start.clone();
            self.start.number += 1;
            Some(tmp)
        } else {
            None
        }
    }
}

pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let tmp = self.start.clone();
            self.start.number += 1;
            Some(tmp)
        } else {
            None
        }
    }
}


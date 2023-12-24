use multiboot2::{BootInformation, MemoryArea};
use super::translation::Frame;

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub struct AreaFrameAllocator<'a> {
    next_free_frame: Frame,
    current_area: Option<&'a MemoryArea>,
    boot_info : &'a BootInformation,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl<'a> AreaFrameAllocator<'a> {

    pub fn new(kernel_start: usize, kernel_end: usize,
               multiboot_start: usize, multiboot_end: usize,
               boot_info_: &'a BootInformation) -> AreaFrameAllocator<'a>
    {
       
        let mut next_free_frame = Frame::containing_address(0);

        let mmap_tag = boot_info_.memory_map_tag();
        let areas = mmap_tag.unwrap().memory_areas();
        let current_area = areas.filter(|area| {
            let address = area.start_address() + area.size() - 1;
            Frame::containing_address(address as usize) >= next_free_frame
        }).min_by_key(|area| area.start_address());

        if let Some(area) = current_area {
            let start_frame = Frame::containing_address(area.start_address() as usize);
            if next_free_frame < start_frame {
                next_free_frame = start_frame;
            }
        }

        AreaFrameAllocator {
            next_free_frame : next_free_frame,
            current_area: current_area,
            boot_info: boot_info_,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
        }
    }

    fn choose_next_area(&mut self) {
        let mmap_tag = self.boot_info.memory_map_tag();
        let areas = mmap_tag.unwrap().memory_areas();
        self.current_area = areas.filter(|area| {
            let address = area.start_address() + area.size() - 1;
            Frame::containing_address(address as usize) >= self.next_free_frame
        }).min_by_key(|area| area.start_address());

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.start_address() as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl<'a> FrameAllocator for AreaFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            let frame = Frame { number: self.next_free_frame.number };

            let current_area_last_frame = {
                let address = area.start_address() + area.size() - 1;
                Frame::containing_address(address as usize)
            };

            if frame > current_area_last_frame {
                self.choose_next_area(); 
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                self.next_free_frame = Frame {
                    number: self.multiboot_end.number + 1
                };
            } else {
                self.next_free_frame.number += 1;
                return Some(frame);
            }
            self.allocate_frame()
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        unimplemented!(); 
    }
}


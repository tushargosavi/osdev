use memory::{Frame, FrameAllocator};
use multiboot2::{MemoryAreaIter, MemoryArea};

pub struct AreaFrameAllocator {
    next_free_frame : Frame,
    current_area : Option<&'static MemoryArea>,
    areas : MemoryAreaIter,
    kernel_start : Frame,
    kernel_end : Frame,
    multiboot_start : Frame,
    multiboot_end : Frame,
}


impl AreaFrameAllocator {

  pub fn new(kernel_start : usize, kernel_end : usize,
         multiboot_start : usize, multiboot_end : usize,
         memory_areas : MemoryAreaIter) -> AreaFrameAllocator {
    let mut allocator = AreaFrameAllocator {
      next_free_frame : Frame::containing_addr(0 as usize),
      current_area : None,
      areas : memory_areas,
      kernel_start : Frame::containing_addr(kernel_start),
      kernel_end : Frame::containing_addr(kernel_end),
      multiboot_start : Frame::containing_addr(multiboot_start),
      multiboot_end : Frame::containing_addr(multiboot_end),
    };
    allocator.choose_next_area();
    allocator
  }

  fn choose_next_area(&mut self) {
    self.current_area = self.areas.clone().filter(|a| {
      let address = a.base_addr + a.length -1;
      Frame::containing_addr(address as usize) >= self.next_free_frame
    }).min_by_key(|a| a.base_addr);

     if let Some(area) = self.current_area {
       let start_frame = Frame::containing_addr(area.base_addr as usize);
       if self.next_free_frame < start_frame {
         self.next_free_frame = start_frame;
       }
     }
  }
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        // save the frame to return later
        if let Some(area) = self.current_area {
           let frame = Frame { number : self.next_free_frame.number };

           let current_area_last_frame = {
             let address = area.base_addr + area.length - 1;
             Frame::containing_addr(address as usize)
           };

           if frame > current_area_last_frame {
             self.choose_next_area()
           } else if frame >= self.kernel_start && frame <= self.kernel_end {
             self.next_free_frame = Frame {
               number : self.kernel_end.number + 1
             };
           } else if frame > self.multiboot_start && frame <= self.multiboot_end {
             self.next_free_frame = Frame {
               number: self.multiboot_end.number + 1
             };
           } else {
             self.next_free_frame.number += 1;
             return Some(frame);
          }
          self.allocate_frame()
        } else {
            None // No frame left
        }
    }

    fn deallocate_frame(&mut self, frame : Frame) {
    }
}

use super::Page;
use super::{ActivePageTable, VirtualAddress};
use memory::Frame;
use memory::FrameAllocator;
use super::table::{Table, Level1};

pub struct TempPage {
  page : Page,
  allocator : TinyAllocator,
}

struct TinyAllocator([Option<Frame>; 3]);

impl TinyAllocator {
  fn new<A>(allocator: &mut A) -> TinyAllocator
    where A: FrameAllocator
  {
    let mut f = || allocator.allocate_frame();
    let frames = [f(), f(), f()];
    TinyAllocator(frames)
  }
}

impl FrameAllocator for TinyAllocator {
  fn allocate_frame(&mut self) -> Option<Frame> {
    for frame_option in &mut self.0 {
      if frame_option.is_some() {
        return frame_option.take();
      }
    }
    None
  }

  fn deallocate_frame(&mut self, frame: Frame) {
    for frame_option in &mut self.0 {
      if frame_option.is_none() {
        *frame_option = Some(frame);
        return
      }
    }
    panic!("TinyAllocator can hold only 3 frames");
  }
}

impl TempPage {
  pub fn map(&mut self, frame : Frame, active_table : &mut ActivePageTable) -> VirtualAddress {
    use super::entry::WRITEABLE;

    assert!(active_table.translate_page(self.page).is_none(),
      "temp page already mapped ");
    active_table.map_to(self.page, frame, WRITEABLE, &mut self.allocator);
    self.page.start_address()
  }

  pub fn unmap(&mut self, active_table : &mut ActivePageTable) {
    active_table.unmap(self.page, &mut self.allocator);
  }

  pub fn map_table_frame(&mut self, frame : Frame, active_table : &mut ActivePageTable) -> &mut Table<Level1> {
    unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>)}
  }
}
use memory::paging::entry::*;
use memory::paging::{Page,ENTRY_COUNT};
use core::ops::{Index, IndexMut};
use memory::FrameAllocator;

pub const P4 : *mut Table<Level4> = 0xffff_ffff_ffff_f000 as *mut _;

pub trait TableLevel {
  fn level() -> u8;
}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {
  fn level() -> u8 { 4 }
}

impl TableLevel for Level3 {
  fn level() -> u8 { 3 }
}
impl TableLevel for Level2 {
  fn level() -> u8 { 2 }
}
impl TableLevel for Level1 {
  fn level() -> u8 { 1 }
}

pub trait HierarchicalLevel : TableLevel {
  type NextLevel : TableLevel;
}

impl HierarchicalLevel for Level4 {
  type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
  type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
  type NextLevel = Level1;
}

use core::marker::PhantomData;

pub struct Table<T> {
  entries : [Entry; ENTRY_COUNT],
  phantomLevel: PhantomData<T>,
}

impl<L> Index<usize> for Table<L> where L : TableLevel {
  type Output = Entry;

  fn index(&self, index : usize) -> &Entry {
    &self.entries[index]
  }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel {
  fn index_mut(&mut self, index : usize) -> &mut Entry {
    &mut self.entries[index]
  }
}


impl<L> Table<L> where L : TableLevel {
  pub fn zero(&mut self) {
    for entry in self.entries.iter_mut() {
      entry.set_unused();
    }
  }

  pub fn level(&self) -> u8 {
    L::level()
  }
}


impl<L> Table<L> where L : HierarchicalLevel {
  fn next_table_address(&self, index : usize) -> Option<usize> {
    let entry_flags = self[index].flags();
    //println!("flags are level {} index {} flags {:?} ", self.level(), index, entry_flags);
    if entry_flags.contains(PRESENT) && !entry_flags.contains(HUGE_PAGE) {
      let table_address = self as *const _ as usize;
      Some((table_address << 9) | (index << 12))
    } else {
      //println!("not found next table address at index 0x{:X}", index);
      None
    }
  }

  pub fn next_table(&self, index : usize) -> Option<&Table<L::NextLevel>> {
    self.next_table_address(index).map(|addr| unsafe { &*(addr as *const _)})
  }

  pub fn next_table_mut(&self, index : usize) -> Option<&mut Table<L::NextLevel>> {
    self.next_table_address(index).map(|addr| unsafe { &mut *(addr as *mut _)})
  }
  
  pub fn next_table_create<A>(&mut self,
                           index : usize,
                           allocator: &mut A) -> &mut Table<L::NextLevel>
                           where A : FrameAllocator {
    if self.next_table(index).is_none() {
      let frame = allocator.allocate_frame().expect("no frame available");
      self.entries[index].set(frame, PRESENT | WRITEABLE);
      self.next_table_mut(index).unwrap().zero();
    }
    self.next_table_mut(index).unwrap()
  }
}


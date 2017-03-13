use memory::paging::entry::*;
use memory::paging::ENTRY_COUNT;
use core::ops::{Index, IndexMut};

pub const P4 : *mut Table = 0xffff_ffff_ffff_f000 as *mut _;

pub struct Table {
  entries : [Entry; ENTRY_COUNT],
}

impl Index<usize> for Table {
  type Output = Entry;

  fn index(&self, index : usize) -> &Entry {
    &self.entries[index]
  }

}

impl IndexMut<usize> for Table {
  fn index_mut(&mut self, index : usize) -> &mut Entry {
    &mut self.entries[index]
  }
}

impl Table {
  pub fn zero(&mut self) {
    for entry in self.entries.iter_mut() {
      entry.set_unused();
    }
  }

  fn next_table_address(&self, index : usize) -> Option<usize> {
    let entry_flags = self[index].flags();
    if entry_flags.contains(PRESENT) && !entry_flags.contains(HUGE_PAGE) {
      let table_address = self as *const _ as usize;
      Some((table_address << 9) | (index << 12))
    } else {
      None
    }
  }

  pub fn next_table(&self, index : usize) -> Option<&Table> {
    self.next_table_address(index).map(|addr| unsafe { &*(addr as *const _)})
  }

  pub fn next_table_mut(&self, index : usize) -> Option<&mut Table> {
    self.next_table_address(index).map(|addr| unsafe { &mut *(addr as *mut _)})
  }
}

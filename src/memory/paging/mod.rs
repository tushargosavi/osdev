use memory::PAGE_SIZE;

const ENTRY_COUNT : usize = 512;

mod entry;
mod table;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub struct Page {
    number : usize,
}


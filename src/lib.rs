#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;

#[macro_use]
mod vga_buffer;
mod memory;

use memory::FrameAllocator;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
  loop {}
}

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
  vga_buffer::clear_screen();
  let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
  let memory_map_tag = boot_info.memory_map_tag()
    .expect("Memory map tag required");

  println!("memory areas:");
  for area in memory_map_tag.memory_areas() {
    println!("    start: 0x{:x}, length: 0x{:x}",
        area.base_addr, area.length);
  }

  let elf_sections_tag = boot_info.elf_sections_tag()
    .expect("ELF section tag required ");

  println!("kernel sections:");
  for section in elf_sections_tag.sections() {
    println!(" addr: 0x{:X}, size 0x{:X}, flags 0x{:x}",
             section.addr, section.size, section.flags);
  }
  
  let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
  let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();
  println!("kernel start 0x{:x} end 0x{:x}", kernel_start, kernel_end);

  let multiboot_start = multiboot_information_address;
  let multiboot_end = multiboot_start + (boot_info.total_size as usize);
  println!("mutliboot start 0x{:x} end 0x{:x}", multiboot_start, multiboot_end);

  let mut frame_allocator = memory::AreaFrameAllocator::new(
    kernel_start as usize, kernel_end as usize, multiboot_start,
    multiboot_end, memory_map_tag.memory_areas());
  println!("{:?}", frame_allocator.allocate_frame());
  for i in 0.. {
    if let None = frame_allocator.allocate_frame() {
        println!("allocated {} frames", i);
        break;
    }
}
  loop {}
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle]
pub extern fn panic_fmt(fmt : core::fmt::Arguments, file: &'static str, line : u32) -> !
{
  println!("\n\nPANIC in {} at line {}:", file, line);
  println!("     {}", fmt);
  loop{}
}

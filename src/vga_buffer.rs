
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
  const fn new(fg: Color, bg: Color) -> ColorCode {
    ColorCode((bg as u8) << 4 | (fg as u8))
  }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ScreenChar {
  ascii_char : u8,
  color : ColorCode,
}


const BUFFER_HEIGHT : usize = 25;
const BUFFER_WIDTH  : usize = 80;

use volatile::Volatile;

struct Buffer {
  chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

use core::ptr::Unique;

pub struct Writer {
   col_pos : usize,
   color_code : ColorCode,
   buffer: Unique<Buffer>,
}

impl Writer {

  fn newline(&mut self) {
    for row in 1..BUFFER_HEIGHT {
      for col in 0..BUFFER_WIDTH {
        let buffer = self.buffer();
        let character = buffer.chars[row][col].read();
        buffer.chars[row-1][col].write(character);
      }
    }
    self.clear_row(BUFFER_HEIGHT - 1);
    self.col_pos = 0;
  }

  fn clear_row(&mut self, row : usize) {
    let blank = ScreenChar {
      ascii_char : b' ',
      color : ColorCode::new(Color::White, Color::Black),
    };
    let buffer = self.buffer();
    for col in 0..BUFFER_WIDTH {
      buffer.chars[row][col].write(blank);
    }
  }

  fn buffer(&mut self) -> &mut Buffer {
    unsafe { self.buffer.get_mut() }
  }

  pub fn write_byte(&mut self, byte : u8) {
    match byte {
       b'\n' => self.newline(),
       byte => {
         if (self.col_pos >= BUFFER_WIDTH) {
           self.newline();
         }

         let row = BUFFER_HEIGHT - 1;
         let col = self.col_pos;

         let cc = self.color_code;
         self.buffer().chars[row][col].write(ScreenChar {
           ascii_char: byte,
           color : cc
         });
         self.col_pos +=1;
       }
    }
  }
}

use core::fmt;

impl fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for byte in s.bytes() {
      self.write_byte(byte);
    }
    Ok(())
  }
}

pub fn print_something() {
  use core::fmt::Write;
  let mut writer = Writer {
    col_pos : 0,
    color_code : ColorCode::new(Color::LightGreen, Color::Black),
    buffer: unsafe { Unique::new(0xb8000 as *mut _) },
  };
  writer.write_byte(b'H');
  writer.write_str("ello!\n");
  write!(writer, "The numbers are {} and {}", 42, 1.0/3.0);
}

pub static mut VGA : Writer = Writer {
  col_pos : 0,
  color_code : ColorCode::new(Color::White, Color::Black),
  buffer: unsafe { Unique::new(0xb8000 as *mut _) },
};

use core::fmt;

// TEAM_259: VGA Text Mode driver for x86_64 visual feedback.

const VGA_PHYS: usize = 0xB8000;

fn vga_buffer() -> *mut u16 {
    crate::x86_64::mem::mmu::phys_to_virt(VGA_PHYS) as *mut u16
}
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    #[allow(dead_code)]
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

pub struct VgaWriter {
    column_position: usize,
    color_code: ColorCode,
}

impl VgaWriter {
    pub const fn new() -> Self {
        Self {
            column_position: 0,
            color_code: ColorCode((Color::Black as u8) << 4 | (Color::White as u8)),
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                unsafe {
                    vga_buffer()
                        .add(row * BUFFER_WIDTH + col)
                        .write_volatile(u16::from(byte) | (u16::from(color_code.0) << 8));
                }
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    let character = vga_buffer().add(row * BUFFER_WIDTH + col).read_volatile();
                    vga_buffer()
                        .add((row - 1) * BUFFER_WIDTH + col)
                        .write_volatile(character);
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = u16::from(b' ') | (u16::from(self.color_code.0) << 8);
        for col in 0..BUFFER_WIDTH {
            unsafe {
                vga_buffer()
                    .add(row * BUFFER_WIDTH + col)
                    .write_volatile(blank);
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Global VGA writer instance.
pub static VGA_WRITER: los_utils::Mutex<VgaWriter> = los_utils::Mutex::new(VgaWriter::new());

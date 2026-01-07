use core::arch::asm;
use core::fmt;

// TEAM_259: Simple Serial Port (COM1) driver for x86_64 early logging.

const COM1: u16 = 0x3F8;

pub struct SerialPort {
    base_port: u16,
}

impl SerialPort {
    pub const fn new(base_port: u16) -> Self {
        Self { base_port }
    }

    /// SAFETY: Must be called with a valid base port.
    pub unsafe fn init(&self) {
        unsafe {
            // Disable interrupts
            self.outb(1, 0x00);
            // Enable DLAB (set baud rate divisor)
            self.outb(3, 0x80);
            // Set divisor to 3 (lo byte) 38400 baud
            self.outb(0, 0x03);
            // (hi byte)
            self.outb(1, 0x00);
            // 8 bits, no parity, one stop bit
            self.outb(3, 0x03);
            // Enable FIFO, clear them, with 14-byte threshold
            self.outb(2, 0xC7);
            // IRQs enabled, RTS/DSR set
            self.outb(4, 0x0B);
        }
    }

    fn line_status(&self) -> u8 {
        unsafe { self.inb(5) }
    }

    fn is_transmit_empty(&self) -> bool {
        (self.line_status() & 0x20) != 0
    }

    pub fn send(&self, data: u8) {
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        unsafe {
            self.outb(0, data);
        }
    }

    pub fn receive(&self) -> u8 {
        while (self.line_status() & 1) == 0 {
            core::hint::spin_loop();
        }
        unsafe { self.inb(0) }
    }

    pub fn read_byte(&self) -> Option<u8> {
        if (self.line_status() & 1) != 0 {
            Some(unsafe { self.inb(0) })
        } else {
            None
        }
    }

    unsafe fn outb(&self, offset: u16, data: u8) {
        unsafe {
            asm!("out dx, al", in("dx") self.base_port + offset, in("al") data, options(nomem, nostack, preserves_flags));
        }
    }

    unsafe fn inb(&self, offset: u16) -> u8 {
        let res: u8;
        unsafe {
            asm!("in al, dx", out("al") res, in("dx") self.base_port + offset, options(nomem, nostack, preserves_flags));
        }
        res
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

// COM1_PORT removed: Replace with console::WRITER (IrqSafeLock) to avoid deadlocks

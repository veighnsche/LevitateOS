//! x86_64 Port I/O abstractions with mock support for unit tests.
//! TEAM_373: Centralized I/O helpers to enable unit testing of hardware drivers.

// =============================================================================
// Real implementation for bare metal (no_std)
// =============================================================================

#[cfg(not(feature = "std"))]
mod real_impl {
    use core::arch::asm;

    /// Output a byte to a port
    #[inline(always)]
    pub unsafe fn outb(port: u16, data: u8) {
        asm!("out dx, al", in("dx") port, in("al") data, options(nomem, nostack, preserves_flags));
    }

    /// Input a byte from a port
    #[inline(always)]
    pub unsafe fn inb(port: u16) -> u8 {
        let res: u8;
        asm!("in al, dx", out("al") res, in("dx") port, options(nomem, nostack, preserves_flags));
        res
    }
}

// =============================================================================
// Mock implementation for std feature (user-space tests)
// =============================================================================

#[cfg(feature = "std")]
mod mock_impl {
    use std::sync::Mutex;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum IoOp {
        Outb(u16, u8),
        Inb(u16),
    }

    // TEAM_373: Use a simple Mutex for recording.
    // In tests, this will be initialized on first access.
    static OPS: Mutex<Vec<IoOp>> = Mutex::new(Vec::new());
    static MOCK_INB: Mutex<Vec<u8>> = Mutex::new(Vec::new());

    /// Output a byte to a port (recorded for verification)
    pub unsafe fn outb(port: u16, data: u8) {
        OPS.lock().unwrap().push(IoOp::Outb(port, data));
        if port == 0x3F8 {
            std::print!("{}", data as char);
        }
    }

    /// Input a byte from a port (returns next value from MOCK_INB)
    pub unsafe fn inb(port: u16) -> u8 {
        OPS.lock().unwrap().push(IoOp::Inb(port));
        let val = MOCK_INB.lock().unwrap().pop();
        match val {
            Some(v) => v,
            None => {
                // Return 0x20 (TX empty) for COM1 status port to prevent infinite loops in tests
                if port == 0x3FD { 0x20 } else { 0 }
            }
        }
    }

    // --- Test Utilities ---

    pub fn clear_ops() {
        OPS.lock().unwrap().clear();
        MOCK_INB.lock().unwrap().clear();
    }

    pub fn get_ops() -> Vec<IoOp> {
        OPS.lock().unwrap().clone()
    }

    pub fn set_mock_inb(values: Vec<u8>) {
        let mut mock = MOCK_INB.lock().unwrap();
        *mock = values;
        mock.reverse(); // So pop() returns them in order
    }
}

// =============================================================================
// Public API
// =============================================================================

#[cfg(not(feature = "std"))]
pub use real_impl::*;

#[cfg(feature = "std")]
pub use mock_impl::*;

use bitflags::bitflags;
use core::ptr::{read_volatile, write_volatile};

bitflags! {
    /// Flag register bits (FR).
    /// Behaviors: [U1] TXFF bit 5, [U2] RXFE bit 4
    pub struct FlagFlags: u32 {
        /// [U1] Transmit FIFO full (bit 5).
        const TXFF = 1 << 5;
        /// [U2] Receive FIFO empty (bit 4).
        const RXFE = 1 << 4;
        /// UART busy.
        const BUSY = 1 << 3;
    }
}

bitflags! {
    /// Control register bits (CR).
    /// Behaviors: [U3] UARTEN bit 0, [U4] TXE bit 8, [U5] RXE bit 9
    pub struct ControlFlags: u32 {
        /// [U3] UART enable (bit 0).
        const UARTEN = 1 << 0;
        /// [U4] Transmit enable (bit 8).
        const TXE    = 1 << 8;
        /// [U5] Receive enable (bit 9).
        const RXE    = 1 << 9;
    }
}

bitflags! {
    /// Line Control register bits (LCR_H).
    /// Behaviors: [U6] FEN bit 4, [U7] WLEN_8 bits 5-6
    pub struct LineControlFlags: u32 {
        /// [U6] Enable FIFOs (bit 4).
        const FEN    = 1 << 4;
        /// [U7] Word length: 8 bits (bits 5-6).
        const WLEN_8 = 0b11 << 5;
    }
}

bitflags! {
    /// Interrupt Mask Set/Clear register (IMSC).
    /// Behaviors: [U8] RXIM bit 4
    pub struct InterruptFlags: u32 {
        /// [U8] Receive interrupt mask (bit 4).
        const RXIM = 1 << 4;
        /// Transmit interrupt mask.
        const TXIM = 1 << 5;
        /// Receive timeout interrupt mask.
        const RTIM = 1 << 6;
    }
}

// TEAM_017: FIFO interrupt level configuration
const IFLS_RX4_8: u32 = 2 << 3; // RX FIFO becomes 4/8 full
const IFLS_TX4_8: u32 = 2 << 0; // TX FIFO becomes 4/8 empty

#[repr(transparent)]
struct Reg<T>(T);

impl<T> Reg<T> {
    fn read(&self) -> T {
        unsafe { read_volatile(&self.0) }
    }
    fn write(&mut self, val: T) {
        unsafe { write_volatile(&mut self.0, val) }
    }
}

#[repr(C)]
struct Registers {
    dr: Reg<u32>,     // 0x00
    rsrecr: Reg<u32>, // 0x04
    _reserved0: [u32; 4],
    fr: Reg<u32>, // 0x18
    _reserved1: u32,
    ilpr: Reg<u32>,  // 0x20
    ibrd: Reg<u32>,  // 0x24
    fbrd: Reg<u32>,  // 0x28
    lcr_h: Reg<u32>, // 0x2C
    cr: Reg<u32>,    // 0x30
    ifls: Reg<u32>,  // 0x34
    imsc: Reg<u32>,  // 0x38
    ris: Reg<u32>,   // 0x3C
    mis: Reg<u32>,   // 0x40
    icr: Reg<u32>,   // 0x44
}

pub struct Pl011Uart {
    base: usize,
}

impl Pl011Uart {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    fn regs(&self) -> &Registers {
        unsafe { &*(self.base as *const Registers) }
    }

    fn regs_mut(&mut self) -> &mut Registers {
        unsafe { &mut *(self.base as *mut Registers) }
    }

    pub fn init(&mut self) {
        // 1. Disable UART
        self.regs_mut().cr.write(0);

        // 2. Clear interrupts
        self.regs_mut().icr.write(0x7FF);

        // 3. Configure FIFO interrupt levels (TEAM_017)
        self.regs_mut().ifls.write(IFLS_RX4_8 | IFLS_TX4_8);

        // 4. Line Control (8n1, FIFOs enabled)
        self.regs_mut()
            .lcr_h
            .write((LineControlFlags::WLEN_8 | LineControlFlags::FEN).bits());

        // 5. Enable UART, TX, RX
        self.regs_mut()
            .cr
            .write((ControlFlags::UARTEN | ControlFlags::TXE | ControlFlags::RXE).bits());

        // 6. Drain stale data from RX FIFO (TEAM_017)
        self.drain_fifo();
    }

    /// Drain any stale bytes from the RX FIFO.
    /// TEAM_017: Matches Redox drain_fifo() behavior.
    pub fn drain_fifo(&mut self) {
        for _ in 0..32 {
            if FlagFlags::from_bits_truncate(self.regs().fr.read()).contains(FlagFlags::RXFE) {
                break;
            }
            let _ = self.regs().dr.read();
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        // Wait while TX FIFO is full
        while FlagFlags::from_bits_truncate(self.regs().fr.read()).contains(FlagFlags::TXFF) {
            core::hint::spin_loop();
        }
        self.regs_mut().dr.write(byte as u32);
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if FlagFlags::from_bits_truncate(self.regs().fr.read()).contains(FlagFlags::RXFE) {
            None
        } else {
            Some(self.regs().dr.read() as u8)
        }
    }

    pub fn enable_rx_interrupt(&mut self) {
        let mut imsc = InterruptFlags::from_bits_truncate(self.regs().imsc.read());
        imsc.insert(InterruptFlags::RXIM);
        imsc.insert(InterruptFlags::RTIM);
        self.regs_mut().imsc.write(imsc.bits());
    }

    pub fn clear_interrupts(&mut self) {
        self.regs_mut().icr.write(0x7FF);
    }
}

// ============================================================================
// Unit Tests - TEAM_030: U1-U8 Bitflags behavior tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // U1: FlagFlags TXFF bit is bit 5
    #[test]
    fn test_flag_flags_txff_bit_position() {
        assert_eq!(FlagFlags::TXFF.bits(), 1 << 5);
    }

    // U2: FlagFlags RXFE bit is bit 4
    #[test]
    fn test_flag_flags_rxfe_bit_position() {
        assert_eq!(FlagFlags::RXFE.bits(), 1 << 4);
    }

    // U3: ControlFlags UARTEN bit is bit 0
    #[test]
    fn test_control_flags_uarten_bit_position() {
        assert_eq!(ControlFlags::UARTEN.bits(), 1 << 0);
    }

    // U4: ControlFlags TXE bit is bit 8
    #[test]
    fn test_control_flags_txe_bit_position() {
        assert_eq!(ControlFlags::TXE.bits(), 1 << 8);
    }

    // U5: ControlFlags RXE bit is bit 9
    #[test]
    fn test_control_flags_rxe_bit_position() {
        assert_eq!(ControlFlags::RXE.bits(), 1 << 9);
    }

    // U6: LineControlFlags FEN bit is bit 4
    #[test]
    fn test_line_control_flags_fen_bit_position() {
        assert_eq!(LineControlFlags::FEN.bits(), 1 << 4);
    }

    // U7: LineControlFlags WLEN_8 is bits 5-6 (0b11 << 5 = 0x60)
    #[test]
    fn test_line_control_flags_wlen8_bit_position() {
        assert_eq!(LineControlFlags::WLEN_8.bits(), 0b11 << 5);
        assert_eq!(LineControlFlags::WLEN_8.bits(), 0x60);
    }

    // U8: InterruptFlags RXIM bit is bit 4
    #[test]
    fn test_interrupt_flags_rxim_bit_position() {
        assert_eq!(InterruptFlags::RXIM.bits(), 1 << 4);
    }
}

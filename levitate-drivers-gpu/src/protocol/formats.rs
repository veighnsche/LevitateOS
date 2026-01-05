//! VirtIO GPU Pixel Formats
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! Pixel format definitions per VirtIO 1.1 Section 5.7.6.8.

/// Pixel formats supported by VirtIO GPU.
///
/// Format naming: Component order from low to high memory address.
/// UNORM = unsigned normalized (0-255 maps to 0.0-1.0).
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Blue, Green, Red, Alpha (QEMU default).
    B8G8R8A8Unorm = 1,
    /// Blue, Green, Red, padding.
    B8G8R8X8Unorm = 2,
    /// Alpha, Red, Green, Blue.
    A8R8G8B8Unorm = 3,
    /// Padding, Red, Green, Blue.
    X8R8G8B8Unorm = 4,
    /// Red, Green, Blue, Alpha.
    R8G8B8A8Unorm = 67,
    /// Padding, Blue, Green, Red.
    X8B8G8R8Unorm = 68,
    /// Alpha, Blue, Green, Red.
    A8B8G8R8Unorm = 121,
    /// Red, Green, Blue, padding.
    R8G8B8X8Unorm = 134,
}

impl Format {
    /// Get the number of bytes per pixel.
    pub const fn bytes_per_pixel(self) -> u32 {
        4 // All formats are 32-bit
    }

    /// Calculate the size in bytes for a given resolution.
    pub const fn buffer_size(self, width: u32, height: u32) -> usize {
        (width * height * self.bytes_per_pixel()) as usize
    }

    /// The default format used by QEMU.
    pub const fn qemu_default() -> Self {
        Self::B8G8R8A8Unorm
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::qemu_default()
    }
}

//! Generic UEFI GOP Framebuffer Driver for LevitateOS
//!
//! This driver provides basic display support using the Limine GOP framebuffer.

#![no_std]

use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;

/// Pixel format for framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGB (red at lowest address)
    Rgb,
    /// BGR (blue at lowest address)
    Bgr,
    /// Unknown format
    Unknown,
}

/// Simple framebuffer-based GPU.
pub struct SimpleGpu {
    address: usize,
    width: u32,
    height: u32,
    pitch: u32,
    format: PixelFormat,
}

impl SimpleGpu {
    /// Create a new simple GPU from framebuffer parameters.
    pub fn new(address: usize, width: u32, height: u32, pitch: u32, format: PixelFormat) -> Self {
        Self {
            address,
            width,
            height,
            pitch,
            format,
        }
    }

    /// Get display resolution
    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get mutable reference to framebuffer memory.
    pub fn framebuffer(&mut self) -> &mut [u8] {
        let size = (self.pitch as usize) * (self.height as usize);
        // SAFETY: The address must be provided by a trusted bootloader (Limine).
        unsafe { core::slice::from_raw_parts_mut(self.address as *mut u8, size) }
    }
}

/// DrawTarget adapter for SimpleGpu.
pub struct SimpleDisplay<'a> {
    gpu: &'a mut SimpleGpu,
}

impl<'a> SimpleDisplay<'a> {
    pub fn new(gpu: &'a mut SimpleGpu) -> Self {
        Self { gpu }
    }
}

impl DrawTarget for SimpleDisplay<'_> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let width = self.gpu.width;
        let height = self.gpu.height;
        let pitch = self.gpu.pitch as usize;
        let is_bgr = matches!(self.gpu.format, PixelFormat::Bgr);
        let fb = self.gpu.framebuffer();

        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.x < width as i32 && point.y >= 0 && point.y < height as i32 {
                let offset = (point.y as usize) * pitch + (point.x as usize) * 4;
                if offset + 3 < fb.len() {
                    if is_bgr {
                        fb[offset] = color.b();
                        fb[offset + 1] = color.g();
                        fb[offset + 2] = color.r();
                    } else {
                        fb[offset] = color.r();
                        fb[offset + 1] = color.g();
                        fb[offset + 2] = color.b();
                    }
                    fb[offset + 3] = 255; // Alpha
                }
            }
        }
        Ok(())
    }
}

impl OriginDimensions for SimpleDisplay<'_> {
    fn size(&self) -> Size {
        Size::new(self.gpu.width, self.gpu.height)
    }
}

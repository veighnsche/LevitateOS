//! GPU Driver for LevitateOS
//!
//! TEAM_114: Wrapper around virtio-drivers VirtIOGpu with embedded-graphics support.
//!
//! This crate provides:
//! - VirtIO GPU initialization via PCI transport
//! - Framebuffer management
//! - embedded-graphics DrawTarget implementation

#![no_std]
#![allow(clippy::unwrap_used)]

use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use levitate_hal::serial_println;
use levitate_pci::PciTransport;
use virtio_drivers::Hal;
use virtio_drivers::device::gpu::VirtIOGpu;

/// GPU error type
#[derive(Debug)]
pub enum GpuError {
    /// PCI device not found
    NotFound,
    /// VirtIO driver error
    VirtioError,
    /// Framebuffer not available
    NoFramebuffer,
}

/// GPU driver wrapper around virtio-drivers VirtIOGpu
pub struct Gpu<H: Hal> {
    inner: VirtIOGpu<H, PciTransport>,
    width: u32,
    height: u32,
    fb_ptr: Option<*mut u8>,
    fb_size: usize,
}

// SAFETY: GPU access should be protected by a lock at the kernel level
unsafe impl<H: Hal> Send for Gpu<H> {}
unsafe impl<H: Hal> Sync for Gpu<H> {}

impl<H: Hal> Gpu<H> {
    /// Create a new GPU driver from a PCI transport
    pub fn new(transport: PciTransport) -> Result<Self, GpuError> {
        let mut gpu = VirtIOGpu::new(transport).map_err(|_| GpuError::VirtioError)?;

        let (width, height) = gpu.resolution().map_err(|_| GpuError::VirtioError)?;
        serial_println!("[GPU] Resolution: {}x{}", width, height);

        // Setup framebuffer
        let fb = gpu.setup_framebuffer().map_err(|_| GpuError::VirtioError)?;
        let fb_ptr = fb.as_mut_ptr();
        let fb_size = fb.len();

        // TEAM_116: Clear to black for terminal background
        for i in (0..fb_size).step_by(4) {
            fb[i] = 0x00; // B
            fb[i + 1] = 0x00; // G
            fb[i + 2] = 0x00; // R
            fb[i + 3] = 0xFF; // A
        }

        // Flush to display
        gpu.flush().map_err(|_| GpuError::VirtioError)?;

        Ok(Self {
            inner: gpu,
            width: width as u32,
            height: height as u32,
            fb_ptr: Some(fb_ptr),
            fb_size,
        })
    }

    /// Flush framebuffer to display
    pub fn flush(&mut self) -> Result<(), GpuError> {
        self.inner.flush().map_err(|_| GpuError::VirtioError)
    }

    /// Get display resolution
    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get mutable reference to framebuffer
    pub fn framebuffer(&mut self) -> &mut [u8] {
        if let Some(ptr) = self.fb_ptr {
            // SAFETY: We own this framebuffer memory
            unsafe { core::slice::from_raw_parts_mut(ptr, self.fb_size) }
        } else {
            &mut []
        }
    }
}

/// Display adapter for embedded-graphics
pub struct Display<'a, H: Hal> {
    gpu: &'a mut Gpu<H>,
}

impl<'a, H: Hal> Display<'a, H> {
    /// Create a new display adapter
    pub fn new(gpu: &'a mut Gpu<H>) -> Self {
        Self { gpu }
    }
}

impl<H: Hal> DrawTarget for Display<'_, H> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let (width, height) = self.gpu.resolution();
        let fb = self.gpu.framebuffer();

        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.x < width as i32 && point.y >= 0 && point.y < height as i32 {
                let idx = (point.y as usize * width as usize + point.x as usize) * 4;
                if idx + 3 < fb.len() {
                    fb[idx] = color.b();
                    fb[idx + 1] = color.g();
                    fb[idx + 2] = color.r();
                    fb[idx + 3] = 255;
                }
            }
        }
        Ok(())
    }
}

impl<H: Hal> OriginDimensions for Display<'_, H> {
    fn size(&self) -> Size {
        let (w, h) = self.gpu.resolution();
        Size::new(w, h)
    }
}

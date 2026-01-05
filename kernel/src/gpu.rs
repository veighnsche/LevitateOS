//! VirtIO GPU Driver
//!
//! TEAM_032: Updated for virtio-drivers v0.12.0
//! - Uses StaticMmioTransport for 'static lifetime compatibility
//!
//! TEAM_065: Fixed error handling per Rule 14 (Fail Loud, Fail Fast)
//! - GPU errors are logged, not silently swallowed
//! - DrawTarget uses GpuError instead of Infallible

use crate::virtio::{StaticMmioTransport, VirtioHal};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use levitate_hal::IrqSafeLock;
use virtio_drivers::device::gpu::VirtIOGpu;

/// TEAM_065: GPU error type for proper error propagation (Rule 6)
#[derive(Debug, Clone, Copy)]
pub enum GpuError {
    /// GPU not initialized
    NotInitialized,
    /// Flush operation failed
    FlushFailed,
}

// TEAM_032: Use StaticMmioTransport (MmioTransport<'static>) for static storage
pub struct GpuState {
    gpu: VirtIOGpu<VirtioHal, StaticMmioTransport>,
    fb_ptr: usize,
    fb_len: usize,
    width: u32,
    height: u32,
}

pub static GPU: IrqSafeLock<Option<GpuState>> = IrqSafeLock::new(None);

pub fn init(transport: StaticMmioTransport) {
    match VirtIOGpu::<VirtioHal, StaticMmioTransport>::new(transport) {
        Ok(mut gpu) => {
            match gpu.resolution() {
                Ok((width, height)) => {
                    // Setup framebuffer
                    match gpu.setup_framebuffer() {
                        Ok(fb) => {
                            let fb_ptr = fb.as_mut_ptr() as usize;
                            let fb_len = fb.len();

                            *GPU.lock() = Some(GpuState {
                                gpu,
                                fb_ptr,
                                fb_len,
                                width,
                                height,
                            });
                        }
                        Err(_e) => {
                            crate::println!("GPU: Failed to setup framebuffer");
                        }
                    }
                }
                Err(_e) => crate::println!("GPU: Failed to get resolution"),
            }
        }
        Err(_e) => crate::println!("GPU: VirtIOGpu::new failed"),
    }
}

impl GpuState {
    pub fn framebuffer(&mut self) -> &mut [u8] {
        // SAFETY: The framebuffer memory is allocated via DMA and kept alive by the VirtIOGpu instance
        // which lives as long as this GpuState. The pointer and length were obtained validity from setup_framebuffer.
        unsafe { core::slice::from_raw_parts_mut(self.fb_ptr as *mut u8, self.fb_len) }
    }

    /// TEAM_030: Get screen dimensions for input coordinate scaling
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// TEAM_065: Flush framebuffer to display with error logging (Rule 14)
    pub fn flush(&mut self) {
        if let Err(_e) = self.gpu.flush() {
            // Rule 14: Fail Loud - log GPU errors
            // TEAM_083: Use serial_println! to avoid recursive deadlock
            levitate_hal::serial_println!("[GPU] ERROR: Flush failed");
        }
    }
}

// TEAM_058: Static resolution getter for terminal initialization (SC2.1)
/// Get current screen resolution without requiring mutable access
/// Returns None if GPU not initialized (SC2.4)
pub fn get_resolution() -> Option<(u32, u32)> {
    GPU.lock().as_ref().map(|s| (s.width, s.height))
}

pub struct Display;

/// TEAM_065: DrawTarget with proper error type (Rule 6)
impl DrawTarget for Display {
    type Color = Rgb888;
    type Error = GpuError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let mut guard = GPU.lock();
        let state = guard.as_mut().ok_or(GpuError::NotInitialized)?;

        let width = state.width;
        let height = state.height;
        let fb = state.framebuffer();

        let mut updated = false;
        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.x < width as i32 && point.y >= 0 && point.y < height as i32 {
                let idx = (point.y as usize * width as usize + point.x as usize) * 4;
                if idx + 3 < fb.len() {
                    fb[idx] = color.r();
                    fb[idx + 1] = color.g();
                    fb[idx + 2] = color.b();
                    fb[idx + 3] = 255; // Alpha
                    updated = true;
                }
            }
        }
        // TEAM_083: Removed automatic flush in draw_iter to improve performance.
        // Callers must call GPU.lock().unwrap().flush() explicitly.
        Ok(())
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        let guard = GPU.lock();
        if let Some(state) = guard.as_ref() {
            Size::new(state.width, state.height)
        } else {
            Size::new(0, 0)
        }
    }
}

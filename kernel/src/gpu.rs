//! Kernel-side GPU Interface
//!
//! TEAM_114: Wrapper around levitate-gpu crate for kernel use.
//!
//! See: `docs/planning/virtio-pci/` for the implementation plan

use levitate_hal::IrqSafeLock;

use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;

use crate::virtio::VirtioHal;

// Re-export from levitate-gpu
pub use levitate_gpu::GpuError;

/// GPU state wrapper using levitate-gpu
pub struct GpuState {
    inner: levitate_gpu::Gpu<VirtioHal>,
}

// SAFETY: GPU access is protected by IrqSafeLock
unsafe impl Send for GpuState {}
unsafe impl Sync for GpuState {}

impl GpuState {
    pub fn flush(&mut self) -> Result<(), GpuError> {
        self.inner.flush()
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.inner.resolution()
    }

    pub fn framebuffer(&mut self) -> &mut [u8] {
        self.inner.framebuffer()
    }
}

// TEAM_122: Use IrqSafeLock to prevent deadlocks between input::poll and ISR prints
pub static GPU: IrqSafeLock<Option<GpuState>> = IrqSafeLock::new(None);

/// Initialize GPU via PCI transport
/// Note: mmio_base is ignored - we use PCI enumeration instead
#[allow(unused_variables)]
pub fn init(mmio_base: usize) {
    levitate_hal::serial_println!("[GPU] Initializing via PCI...");

    match levitate_pci::find_virtio_gpu::<VirtioHal>() {
        Some(transport) => match levitate_gpu::Gpu::new(transport) {
            Ok(gpu) => {
                levitate_hal::serial_println!("[GPU] Initialized via PCI transport");
                *GPU.lock() = Some(GpuState { inner: gpu });
            }
            Err(e) => {
                levitate_hal::serial_println!("[GPU] Failed to create GPU: {:?}", e);
            }
        },
        None => {
            levitate_hal::serial_println!("[GPU] No VirtIO GPU found on PCI bus");
        }
    }
}

pub fn get_resolution() -> Option<(u32, u32)> {
    GPU.lock().as_ref().map(|s| s.dimensions())
}

pub struct Display<'a> {
    pub state: &'a mut GpuState,
}

impl<'a> Display<'a> {
    pub fn new(state: &'a mut GpuState) -> Self {
        Self { state }
    }
}

impl<'a> DrawTarget for Display<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let (width, height) = self.state.dimensions();
        let fb = self.state.framebuffer();

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

impl<'a> OriginDimensions for Display<'a> {
    fn size(&self) -> Size {
        let (w, h) = self.state.dimensions();
        Size::new(w, h)
    }
}

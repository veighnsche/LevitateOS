//! Kernel-side GPU Interface
//!
//! # ⚠️ CRITICAL WARNING FOR AI AGENTS ⚠️
//!
//! **DO NOT "FIX" THIS BY KEEPING levitate-gpu!**
//!
//! The current `levitate-gpu` crate (using virtio-drivers) is **BROKEN**.
//! It gives **FALSE POSITIVE TESTS** - the driver initializes but the
//! display shows NOTHING. The QEMU window says "Display output is not active".
//!
//! ## The Bug You Must Resist
//!
//! The temptation to "fall back" to levitate-gpu when levitate-drivers-gpu
//! has issues is **ITSELF A BUG**. Previous teams kept doing this, which
//! prevented any progress on actually fixing the GPU.
//!
//! ## What You Must Do Instead
//!
//! 1. **FIX levitate-drivers-gpu** - see `docs/VIRTIO_IMPLEMENTATION.md`
//! 2. **DO NOT** revert to levitate-gpu thinking it "works" - it doesn't
//! 3. **DO NOT** trust "GPU initialized successfully" - it's a false positive
//! 4. **VERIFY** actual display output in QEMU window, not just test results
//!
//! ## Why levitate-gpu Doesn't Work
//!
//! - Missing SET_SCANOUT command (required by VirtIO GPU spec)
//! - Missing RESOURCE_FLUSH command
//! - No actual scanout configuration
//! - Tests pass because they only check init, not display output
//!
//! See: `.teams/TEAM_109_fix_gpu_driver_no_fallback.md`
//! See: `docs/GOTCHAS.md` section 16
//! See: `docs/VIRTIO_IMPLEMENTATION.md`

use levitate_drivers_gpu::{GpuError, VirtioGpu};
use levitate_hal::IrqSafeLock;
use levitate_virtio::LevitateVirtioHal;

pub struct GpuState {
    pub inner: VirtioGpu<LevitateVirtioHal>,
}

impl GpuState {
    pub fn new(mmio_base: usize) -> Result<Self, GpuError> {
        let mut inner = unsafe { VirtioGpu::<LevitateVirtioHal>::new(mmio_base)? };
        inner.init()?;
        Ok(Self { inner })
    }

    pub fn flush(&mut self) -> Result<(), GpuError> {
        self.inner.flush()
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.inner.resolution()
    }

    pub fn framebuffer(&mut self) -> &mut [u8] {
        self.inner.framebuffer().unwrap_or(&mut [])
    }
}

pub static GPU: IrqSafeLock<Option<GpuState>> = IrqSafeLock::new(None);

pub fn init(mmio_base: usize) {
    match GpuState::new(mmio_base) {
        Ok(state) => {
            *GPU.lock() = Some(state);
        }
        Err(e) => {
            levitate_hal::serial_println!("[GPU] Init failed: {:?}", e);
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

use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;

impl<'a> DrawTarget for Display<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.state.inner.draw_iter(pixels)
    }
}

impl<'a> OriginDimensions for Display<'a> {
    fn size(&self) -> Size {
        let (w, h) = self.state.dimensions();
        Size::new(w, h)
    }
}

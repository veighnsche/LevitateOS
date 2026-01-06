//! Kernel-side GPU Interface
//!
//! TEAM_114: Wrapper around levitate-gpu crate for kernel use.
//! TEAM_141: Removed duplicate Display type - use levitate_gpu::Display via as_display()
//!
//! See: `docs/planning/virtio-pci/` for the implementation plan

use levitate_hal::IrqSafeLock;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::virtio::VirtioHal;

// Re-export from levitate-gpu
pub use levitate_gpu::{Display, GpuError};

/// GPU state wrapper using levitate-gpu
pub struct GpuState {
    inner: levitate_gpu::Gpu<VirtioHal>,
}

// SAFETY: GPU access is protected by IrqSafeLock
unsafe impl Send for GpuState {}
unsafe impl Sync for GpuState {}

impl GpuState {
    pub fn flush(&mut self) -> Result<(), GpuError> {
        // TEAM_129: Increment flush counter for regression testing
        FLUSH_COUNT.fetch_add(1, Ordering::Relaxed);
        self.inner.flush()
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.inner.resolution()
    }

    pub fn framebuffer(&mut self) -> &mut [u8] {
        self.inner.framebuffer()
    }

    /// TEAM_141: Get a Display adapter for embedded-graphics DrawTarget
    /// Uses levitate_gpu::Display instead of duplicate kernel implementation
    pub fn as_display(&mut self) -> Display<'_, VirtioHal> {
        Display::new(&mut self.inner)
    }
}

// TEAM_122: Use IrqSafeLock to prevent deadlocks between input::poll and ISR prints
pub static GPU: IrqSafeLock<Option<GpuState>> = IrqSafeLock::new(None);

// TEAM_129: Flush counter for regression testing - ensures GPU flush is actually called
static FLUSH_COUNT: AtomicU32 = AtomicU32::new(0);

/// Get the number of times GPU flush has been called (for testing)
pub fn flush_count() -> u32 {
    FLUSH_COUNT.load(Ordering::Relaxed)
}

/// TEAM_129: Check if framebuffer contains any non-black pixels (for regression testing)
/// Returns (total_pixels, non_black_count) to verify terminal actually rendered content
pub fn framebuffer_has_content() -> Option<(usize, usize)> {
    let mut guard = GPU.lock();
    if let Some(gpu_state) = guard.as_mut() {
        let fb = gpu_state.framebuffer();
        let total_pixels = fb.len() / 4; // BGRA format
        let mut non_black = 0usize;
        
        // Sample every 100th pixel for performance (still catches any rendering)
        for i in (0..fb.len()).step_by(400) { // 400 = 100 pixels * 4 bytes
            // Check if R, G, or B is non-zero (index+2=R, index+1=G, index=B)
            if i + 2 < fb.len() && (fb[i] != 0 || fb[i + 1] != 0 || fb[i + 2] != 0) {
                non_black += 1;
            }
        }
        Some((total_pixels, non_black * 100)) // Scale back up
    } else {
        None
    }
}

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

// TEAM_141: Removed duplicate Display type - use levitate_gpu::Display via GpuState::as_display()

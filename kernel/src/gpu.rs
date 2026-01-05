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

// FIXME(GPU): This uses levitate-gpu which gives FALSE POSITIVES!
// The display doesn't actually work. Replace with levitate-drivers-gpu
// once VirtQueue issues are fixed. DO NOT KEEP THIS AS A "WORKING" SOLUTION.
pub use levitate_gpu::{Display, GpuState};
use levitate_hal::{IrqSafeLock, StaticMmioTransport};

pub static GPU: IrqSafeLock<Option<GpuState>> = IrqSafeLock::new(None);

pub fn init(transport: StaticMmioTransport) {
    match GpuState::new(transport) {
        Ok(state) => {
            *GPU.lock() = Some(state);
        }
        Err(e) => {
            levitate_hal::serial_println!("[GPU] Init failed: {}", e);
        }
    }
}

pub fn get_resolution() -> Option<(u32, u32)> {
    GPU.lock().as_ref().map(|s| (s.width, s.height))
}

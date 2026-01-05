//! Kernel-side GPU Interface
//! TEAM_100: Reverted to levitate-gpu (virtio-drivers) for working terminal
//! The new levitate-virtio-gpu driver needs VirtQueue fixes - see docs/planning/virtio-gpu-scanout/

// TEAM_103: GpuError removed (unused), Display/GpuState still needed
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

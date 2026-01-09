//! TEAM_282: Device Tree Blob → BootInfo Parser
//!
//! Converts DTB (Device Tree Blob) information to the unified BootInfo format.
//! Used for AArch64 boot path from QEMU or real ARM hardware.
//!
//! This is a transitional module - will be unified with Limine once implemented.

use super::{BootInfo, BootProtocol, FirmwareInfo, MemoryKind, MemoryRegion};
use los_hal::aarch64::fdt::{self, Fdt};

/// TEAM_282: Parse DTB into BootInfo.
///
/// Extracts memory map and other boot information from Device Tree.
///
/// # Arguments
/// * `dtb_ptr` - Physical address of the DTB
///
/// # Safety
/// The `dtb_ptr` must point to a valid Device Tree Blob.
pub unsafe fn parse(dtb_ptr: usize) -> BootInfo {
    let mut boot_info = BootInfo::empty();
    boot_info.protocol = BootProtocol::DeviceTree;
    boot_info.firmware = FirmwareInfo::DeviceTree { dtb: dtb_ptr };

    // Try to parse the DTB
    // SAFETY: dtb_ptr is provided by the bootloader and is expected to be a valid
    // pointer to a Device Tree Blob in memory.
    let dtb_slice = unsafe { core::slice::from_raw_parts(dtb_ptr as *const u8, 1024 * 1024) };

    if let Ok(fdt_obj) = Fdt::new(dtb_slice) {
        // Extract memory regions using HAL helper
        fdt::for_each_memory_region(&fdt_obj, |region| {
            let _ = boot_info.memory_map.push(MemoryRegion::new(
                region.start,
                region.end - region.start,
                MemoryKind::Usable,
            ));
        });

        // Try to find initramfs
        if let Ok((start, end)) = fdt::get_initrd_range(dtb_slice) {
            if end > start {
                boot_info.initramfs =
                    Some(MemoryRegion::new(start, end - start, MemoryKind::Bootloader));
            }
        }
    }

    boot_info
}

/// TEAM_282: Parse DTB from a slice.
///
/// Alternative parser that takes a pre-validated DTB slice.
pub fn parse_from_slice(dtb_slice: &[u8], dtb_phys: usize) -> BootInfo {
    let mut boot_info = BootInfo::empty();
    boot_info.protocol = BootProtocol::DeviceTree;
    boot_info.firmware = FirmwareInfo::DeviceTree { dtb: dtb_phys };

    if let Ok(fdt_obj) = Fdt::new(dtb_slice) {
        // Extract memory regions using HAL helper
        fdt::for_each_memory_region(&fdt_obj, |region| {
            let _ = boot_info.memory_map.push(MemoryRegion::new(
                region.start,
                region.end - region.start,
                MemoryKind::Usable,
            ));
        });

        // Try to find initramfs
        if let Ok((start, end)) = fdt::get_initrd_range(dtb_slice) {
            if end > start {
                boot_info.initramfs =
                    Some(MemoryRegion::new(start, end - start, MemoryKind::Bootloader));
            }
        }
    }

    boot_info
}

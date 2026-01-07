//! TEAM_282: Multiboot → BootInfo Parser
//!
//! Converts Multiboot1/2 boot information to the unified BootInfo format.
//! This is a transitional module - will be removed once Limine is primary.

use super::{BootInfo, BootProtocol, FirmwareInfo, MemoryKind, MemoryMap, MemoryRegion};

/// Multiboot1 magic value (from QEMU -kernel)
pub const MULTIBOOT1_MAGIC: u32 = 0x2BADB002;

/// Multiboot2 magic value (from GRUB)
pub const MULTIBOOT2_MAGIC: u32 = 0x36D76289;

/// TEAM_282: Parse Multiboot1 info into BootInfo.
///
/// Multiboot1 is simpler - we just extract basic memory info.
/// Used primarily for QEMU -kernel boot path.
///
/// # Safety
/// The `info_ptr` must point to a valid Multiboot1 info structure.
pub unsafe fn parse_multiboot1(magic: u32, info_ptr: usize) -> BootInfo {
    if magic != MULTIBOOT1_MAGIC {
        return BootInfo::empty();
    }

    let mut boot_info = BootInfo::empty();
    boot_info.protocol = BootProtocol::Multiboot1;

    // Multiboot1 info structure (simplified)
    // We only parse the memory map if available
    #[repr(C)]
    struct Multiboot1Info {
        flags: u32,
        mem_lower: u32,
        mem_upper: u32,
        // ... more fields we don't need
    }

    let mb_info = unsafe { &*(info_ptr as *const Multiboot1Info) };

    // Check if basic memory info is valid (bit 0 of flags)
    if mb_info.flags & 0x1 != 0 {
        // mem_lower is in KB, below 1MB
        // mem_upper is in KB, above 1MB
        let lower_size = (mb_info.mem_lower as usize) * 1024;
        let upper_size = (mb_info.mem_upper as usize) * 1024;

        // Add conventional memory (below 1MB, typically 640KB usable)
        if lower_size > 0 {
            let _ = boot_info.memory_map.push(MemoryRegion::new(
                0,
                lower_size.min(0xA0000), // Cap at 640KB (VGA memory starts at 0xA0000)
                MemoryKind::Usable,
            ));
        }

        // Add extended memory (above 1MB)
        if upper_size > 0 {
            let _ = boot_info.memory_map.push(MemoryRegion::new(
                0x100000, // 1MB
                upper_size,
                MemoryKind::Usable,
            ));
        }
    }

    // No ACPI info from Multiboot1 typically
    boot_info.firmware = FirmwareInfo::None;

    boot_info
}

/// TEAM_282: Parse Multiboot2 info into BootInfo.
///
/// Uses the existing HAL multiboot2 parser and converts to BootInfo format.
///
/// # Safety
/// The `info_ptr` must point to a valid Multiboot2 info structure.
pub unsafe fn parse_multiboot2(magic: u32, info_ptr: usize) -> BootInfo {
    if magic != MULTIBOOT2_MAGIC {
        return BootInfo::empty();
    }

    let mut boot_info = BootInfo::empty();
    boot_info.protocol = BootProtocol::Multiboot2;

    // Use the existing HAL parser
    let parsed = unsafe { los_hal::x86_64::multiboot2::parse(info_ptr) };

    // Convert memory regions
    for region_opt in parsed.ram_regions.iter() {
        if let Some(region) = region_opt {
            let kind = match region.typ {
                los_hal::x86_64::multiboot2::MemoryType::Available => MemoryKind::Usable,
                los_hal::x86_64::multiboot2::MemoryType::Reserved => MemoryKind::Reserved,
                los_hal::x86_64::multiboot2::MemoryType::AcpiReclaimable => {
                    MemoryKind::AcpiReclaimable
                }
                los_hal::x86_64::multiboot2::MemoryType::Nvs => MemoryKind::AcpiNvs,
                los_hal::x86_64::multiboot2::MemoryType::BadRam => MemoryKind::BadMemory,
            };

            let _ = boot_info.memory_map.push(MemoryRegion::new(
                region.start,
                region.end - region.start,
                kind,
            ));
        }
    }

    // TODO(TEAM_282): Parse RSDP from multiboot2 ACPI tags
    // For now, no firmware info
    boot_info.firmware = FirmwareInfo::None;

    boot_info
}

/// TEAM_282: Auto-detect and parse multiboot info.
///
/// Detects whether this is Multiboot1 or Multiboot2 based on magic value.
///
/// # Safety
/// The `info_ptr` must point to a valid Multiboot info structure.
pub unsafe fn parse(magic: u32, info_ptr: usize) -> BootInfo {
    match magic {
        MULTIBOOT1_MAGIC => unsafe { parse_multiboot1(magic, info_ptr) },
        MULTIBOOT2_MAGIC => unsafe { parse_multiboot2(magic, info_ptr) },
        _ => {
            // Unknown magic - return empty with Unknown protocol
            let mut info = BootInfo::empty();
            info.protocol = BootProtocol::Unknown;
            info
        }
    }
}

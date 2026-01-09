//! TEAM_282: Limine Boot Protocol Support
//!
//! This module implements support for the Limine boot protocol, which is the
//! primary boot method for LevitateOS on both x86_64 and AArch64.
//!
//! # Features
//! - Modern, well-documented protocol
//! - Kernel enters in 64-bit long mode (no 32→64 transition needed)
//! - Provides: memory map, framebuffer, RSDP, SMP info, modules
//! - Supports both x86_64 AND AArch64
//!
//! # References
//! - https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md
//! - https://crates.io/crates/limine

use super::{
    BootInfo, BootProtocol, Framebuffer, MemoryKind, MemoryRegion, PixelFormat,
};
#[cfg(target_arch = "x86_64")]
use super::FirmwareInfo;
use limine::BaseRevision;
use limine::memory_map::EntryType;
use limine::request::{
    ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest, ModuleRequest,
    RsdpRequest, StackSizeRequest,
};

/// TEAM_282: Limine base revision request.
/// This tells Limine which protocol version we support.
#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

/// TEAM_282: Request for Higher Half Direct Map (HHDM).
/// This gives us the offset to access physical memory from kernel space.
#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// TEAM_282: Request for memory map.
/// Limine provides a detailed memory map of the system.
#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

/// TEAM_282: Request for framebuffer.
/// Optional - for early graphical console.
#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

/// TEAM_282: Request for RSDP (ACPI Root System Description Pointer).
/// Used on x86_64 for ACPI table discovery.
#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

/// TEAM_282: Request for boot modules (initramfs).
#[used]
#[unsafe(link_section = ".requests")]
static MODULE_REQUEST: ModuleRequest = ModuleRequest::new();

/// TEAM_282: Request stack size (64KB minimum).
#[used]
#[unsafe(link_section = ".requests")]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(64 * 1024);

/// TEAM_288: Request for kernel address.
/// Used to get actual physical load address for virt_to_phys calculations.
#[used]
#[unsafe(link_section = ".requests")]
static KERNEL_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

/// TEAM_282: Get the Higher Half Direct Map offset.
///
/// Returns the virtual address offset for accessing physical memory.
pub fn hhdm_offset() -> Option<u64> {
    HHDM_REQUEST.get_response().map(|r| r.offset())
}

/// TEAM_282: Parse Limine boot information into BootInfo.
///
/// This is called during early kernel initialization to convert
/// Limine's response structures into our unified BootInfo format.
pub fn parse() -> BootInfo {
    let mut boot_info = BootInfo::empty();
    boot_info.protocol = BootProtocol::Limine;

    // Check if Limine responded to our base revision request
    // TEAM_286: Even if unsupported, keep protocol as Limine since we know we're
    // booting via Limine (boot.S sets magic=0). This ensures CR3 switch is skipped.
    if !BASE_REVISION.is_supported() {
        // Limine v7 may not fill responses, but we're still booting via Limine
        // Don't return early - try to parse what we can
    }

    // TEAM_288: Set kernel physical base and HHDM offset from Limine if available
    // This fixes virt_to_phys() when Limine loads kernel at different PA than linker assumes
    #[cfg(target_arch = "x86_64")]
    {
        if let Some(addr_response) = KERNEL_ADDRESS_REQUEST.get_response() {
            los_hal::mmu::set_kernel_phys_base(addr_response.physical_base() as usize);
        }
        if let Some(offset) = hhdm_offset() {
            los_hal::mmu::set_phys_offset(offset as usize);
        }
    }

    // Parse memory map
    if let Some(memmap_response) = MEMORY_MAP_REQUEST.get_response() {
        for entry in memmap_response.entries() {
            let kind = match entry.entry_type {
                EntryType::USABLE => MemoryKind::Usable,
                EntryType::RESERVED => MemoryKind::Reserved,
                EntryType::ACPI_RECLAIMABLE => MemoryKind::AcpiReclaimable,
                EntryType::ACPI_NVS => MemoryKind::AcpiNvs,
                EntryType::BAD_MEMORY => MemoryKind::BadMemory,
                EntryType::BOOTLOADER_RECLAIMABLE => MemoryKind::Bootloader,
                EntryType::EXECUTABLE_AND_MODULES => MemoryKind::Kernel,
                EntryType::FRAMEBUFFER => MemoryKind::Framebuffer,
                _ => MemoryKind::Unknown,
            };

            let _ = boot_info.memory_map.push(MemoryRegion::new(
                entry.base as usize,
                entry.length as usize,
                kind,
            ));
        }
    }

    // Parse RSDP for ACPI (x86_64)
    #[cfg(target_arch = "x86_64")]
    if let Some(rsdp_response) = RSDP_REQUEST.get_response() {
        let rsdp_addr = rsdp_response.address() as usize;
        if rsdp_addr != 0 {
            boot_info.firmware = FirmwareInfo::Acpi { rsdp: rsdp_addr };
        }
    }

    // On AArch64, we might get DTB from Limine (if available)
    // For now, leave firmware as None - Limine on AArch64 provides
    // everything we need via other requests
    #[cfg(target_arch = "aarch64")]
    {
        // TODO(TEAM_282): Check if Limine provides DTB on AArch64
        // For now, firmware info stays as None
    }

    // Parse framebuffer
    if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_response.framebuffers().next() {
            let format = match (fb.red_mask_shift(), fb.blue_mask_shift()) {
                (0, 16) => PixelFormat::Rgb,
                (16, 0) => PixelFormat::Bgr,
                _ => PixelFormat::Unknown,
            };

            boot_info.framebuffer = Some(Framebuffer {
                address: fb.addr() as usize,
                width: fb.width() as u32,
                height: fb.height() as u32,
                pitch: fb.pitch() as u32,
                bpp: fb.bpp() as u8,
                format,
            });
        }
    }

    // Parse modules (initramfs)
    if let Some(module_response) = MODULE_REQUEST.get_response() {
        let modules = module_response.modules();
        if let Some(module) = modules.first() {
            let base = module.addr() as *const u8 as usize;
            let size = module.size() as usize;
            if size > 0 {
                boot_info.initramfs = Some(MemoryRegion::new(base, size, MemoryKind::Bootloader));
            }
        }
    }

    boot_info
}

/// TEAM_282: Check if we were booted via Limine.
pub fn is_limine_boot() -> bool {
    BASE_REVISION.is_supported()
}

//! TEAM_282: Boot Abstraction Layer
//!
//! This module translates bootloader-specific information into a unified
//! `BootInfo` struct consumed by the kernel.
//!
//! # Supported Protocols
//! - **Limine** (x86_64, AArch64) - Primary, modern
//! - **Multiboot1/2** (x86_64) - Legacy QEMU support (transitional)
//! - **Device Tree** (AArch64) - ARM standard (transitional)
//!
//! # Architecture
//! ```text
//! ┌─────────────┬─────────────┬─────────────┐
//! │   Limine    │  Multiboot  │     DTB     │
//! └──────┬──────┴──────┬──────┴──────┬──────┘
//!        │             │             │
//!        └─────────────┼─────────────┘
//!                      │
//!                      ▼ parse_*() → BootInfo
//!               ┌─────────────┐
//!               │  BootInfo   │
//!               │   struct    │
//!               └──────┬──────┘
//!                      │
//!                      ▼ unified signature
//!               ┌─────────────┐
//!               │ kernel_main │
//!               │ (&BootInfo) │
//!               └─────────────┘
//! ```

// TEAM_316: Protocol-specific parsers (simplified - Limine only for x86_64)
#[cfg(target_arch = "aarch64")]
pub mod dtb;
pub mod limine;  // TEAM_316: Primary and only boot method for x86_64

/// Maximum number of memory regions we support.
/// This is a compile-time limit to avoid dynamic allocation during early boot.
pub const MAX_MEMORY_REGIONS: usize = 64;

/// TEAM_282: Unified boot information - the ONE interface between bootloader and kernel.
///
/// This struct follows UNIX philosophy:
/// - Rule 2 (Composition): Designed for consumption by any subsystem
/// - Rule 3 (Expressive Types): Type-safe, self-describing, no magic numbers
#[derive(Debug)]
pub struct BootInfo {
    /// Physical memory map - regions available for kernel use
    pub memory_map: MemoryMap,

    /// Framebuffer for early console (optional)
    pub framebuffer: Option<Framebuffer>,

    /// Platform-specific firmware tables
    pub firmware: FirmwareInfo,

    /// Kernel command line
    pub cmdline: Option<&'static str>,

    /// Initial ramdisk location
    pub initramfs: Option<MemoryRegion>,

    /// Boot protocol that was used
    pub protocol: BootProtocol,
}

impl BootInfo {
    /// Create an empty BootInfo (for initialization)
    pub const fn empty() -> Self {
        Self {
            memory_map: MemoryMap::empty(),
            framebuffer: None,
            firmware: FirmwareInfo::None,
            cmdline: None,
            initramfs: None,
            protocol: BootProtocol::Unknown,
        }
    }
}

/// TEAM_282: Memory map - array of typed regions.
///
/// Uses a fixed-size array to avoid heap allocation during early boot.
#[derive(Debug)]
pub struct MemoryMap {
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    count: usize,
}

impl MemoryMap {
    /// Create an empty memory map
    pub const fn empty() -> Self {
        Self {
            regions: [MemoryRegion::EMPTY; MAX_MEMORY_REGIONS],
            count: 0,
        }
    }

    /// Add a region to the memory map
    pub fn push(&mut self, region: MemoryRegion) -> Result<(), &'static str> {
        if self.count >= MAX_MEMORY_REGIONS {
            return Err("Memory map overflow");
        }
        self.regions[self.count] = region;
        self.count += 1;
        Ok(())
    }

    /// Get the number of regions
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Iterate over regions
    pub fn iter(&self) -> impl Iterator<Item = &MemoryRegion> {
        self.regions[..self.count].iter()
    }

    /// Get usable memory regions only
    pub fn usable(&self) -> impl Iterator<Item = &MemoryRegion> {
        self.iter().filter(|r| r.kind == MemoryKind::Usable)
    }

    /// Calculate total usable memory
    pub fn total_usable(&self) -> usize {
        self.usable().map(|r| r.size).sum()
    }
}

/// TEAM_282: A single memory region with type information.
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    /// Physical base address
    pub base: usize,
    /// Size in bytes
    pub size: usize,
    /// Type of memory region
    pub kind: MemoryKind,
}

impl MemoryRegion {
    /// Empty/invalid region constant
    pub const EMPTY: Self = Self {
        base: 0,
        size: 0,
        kind: MemoryKind::Reserved,
    };

    /// Create a new memory region
    pub const fn new(base: usize, size: usize, kind: MemoryKind) -> Self {
        Self { base, size, kind }
    }

    /// Get the end address (exclusive)
    pub const fn end(&self) -> usize {
        self.base + self.size
    }
}

/// TEAM_282: Memory region types.
///
/// These are normalized from bootloader-specific types to a common set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    /// Free RAM for kernel use
    Usable,
    /// Firmware reserved (do not touch)
    Reserved,
    /// Can be reclaimed after ACPI init
    AcpiReclaimable,
    /// ACPI NVS (non-volatile storage)
    AcpiNvs,
    /// Where kernel is loaded
    Kernel,
    /// Bootloader data (can reclaim after boot)
    Bootloader,
    /// Video memory / framebuffer
    Framebuffer,
    /// Bad memory (hardware defect)
    BadMemory,
    /// Unknown type
    Unknown,
}

/// TEAM_282: Platform firmware info - architecture-specific.
///
/// This enum allows arch-specific firmware access while maintaining
/// a unified BootInfo structure.
#[derive(Debug, Clone, Copy)]
pub enum FirmwareInfo {
    /// x86_64: ACPI tables via RSDP pointer
    Acpi {
        /// Physical address of RSDP structure
        rsdp: usize,
    },

    /// AArch64: Device Tree Blob pointer
    DeviceTree {
        /// Physical address of DTB
        dtb: usize,
    },

    /// No firmware info available
    None,
}

impl FirmwareInfo {
    /// Get RSDP address if this is ACPI firmware
    pub fn rsdp(&self) -> Option<usize> {
        match self {
            FirmwareInfo::Acpi { rsdp } => Some(*rsdp),
            _ => None,
        }
    }

    /// Get DTB address if this is DeviceTree firmware
    pub fn dtb(&self) -> Option<usize> {
        match self {
            FirmwareInfo::DeviceTree { dtb } => Some(*dtb),
            _ => None,
        }
    }
}

/// TEAM_316: Boot protocol identifier (simplified).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootProtocol {
    /// Limine boot protocol (x86_64 and AArch64)
    Limine,
    /// Device Tree boot (AArch64 QEMU)
    DeviceTree,
    /// Unknown/undetected
    Unknown,
}

/// TEAM_282: Framebuffer info for early console.
#[derive(Debug, Clone, Copy)]
pub struct Framebuffer {
    /// Physical address of framebuffer memory
    pub address: usize,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Bytes per scanline
    pub pitch: u32,
    /// Bits per pixel
    pub bpp: u8,
    /// Pixel format
    pub format: PixelFormat,
}

/// TEAM_282: Pixel format for framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGB (red at lowest address)
    Rgb,
    /// BGR (blue at lowest address)
    Bgr,
    /// Unknown format
    Unknown,
}

// TEAM_282: Global boot info storage using atomic pointer pattern
// This follows the same approach as los_hal::x86_64::multiboot2
use core::sync::atomic::{AtomicPtr, Ordering};

static BOOT_INFO_PTR: AtomicPtr<BootInfo> = AtomicPtr::new(core::ptr::null_mut());

// Static storage for boot info (initialized once)
static mut BOOT_INFO_STORAGE: BootInfo = BootInfo::empty();

/// TEAM_282: Get reference to the global boot info.
///
/// Returns None if boot info hasn't been initialized yet.
pub fn boot_info() -> Option<&'static BootInfo> {
    let ptr = BOOT_INFO_PTR.load(Ordering::Acquire);
    if ptr.is_null() {
        None
    } else {
        // SAFETY: Once set, BOOT_INFO_STORAGE is never modified again
        unsafe { Some(&*ptr) }
    }
}

/// TEAM_282: Initialize the global boot info.
///
/// # Safety
/// Must only be called once during early boot, before any other code
/// accesses boot_info().
pub unsafe fn set_boot_info(info: BootInfo) {
    // SAFETY: Called only once during single-threaded early boot
    unsafe {
        BOOT_INFO_STORAGE = info;
        BOOT_INFO_PTR.store(
            core::ptr::addr_of_mut!(BOOT_INFO_STORAGE),
            Ordering::Release,
        );
    }
}

//! TEAM_267: Multiboot2 Boot Information Parsing
//!
//! Parses the Multiboot2 boot information structure passed by the bootloader.
//! Reference: https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html

use core::mem;

/// Multiboot2 magic value passed in EAX
pub const MULTIBOOT2_BOOTLOADER_MAGIC: u32 = 0x36d76289;

/// Tag types from Multiboot2 specification
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    End = 0,
    Cmdline = 1,
    BootLoaderName = 2,
    Module = 3,
    BasicMemInfo = 4,
    BiosBootDevice = 5,
    MemoryMap = 6,
    VbeInfo = 7,
    FramebufferInfo = 8,
    ElfSections = 9,
    Apm = 10,
    Efi32 = 11,
    Efi64 = 12,
    Smbios = 13,
    AcpiOld = 14,
    AcpiNew = 15,
    Network = 16,
    EfiMemoryMap = 17,
    EfiBs = 18,
    Efi32ImageHandle = 19,
    Efi64ImageHandle = 20,
    LoadBaseAddr = 21,
}

/// Memory region types from Multiboot2 specification
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    Available = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    Nvs = 4,
    BadRam = 5,
}

impl MemoryType {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => Self::Available,
            2 => Self::Reserved,
            3 => Self::AcpiReclaimable,
            4 => Self::Nvs,
            5 => Self::BadRam,
            _ => Self::Reserved,
        }
    }

    /// Returns true if this memory type is usable as general RAM
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Multiboot2 boot information header
#[repr(C)]
pub struct BootInfo {
    pub total_size: u32,
    pub reserved: u32,
    // Tags follow...
}

/// Generic tag header
#[repr(C)]
pub struct Tag {
    pub typ: u32,
    pub size: u32,
}

/// Memory map tag header
#[repr(C)]
pub struct MemoryMapTag {
    pub typ: u32,
    pub size: u32,
    pub entry_size: u32,
    pub entry_version: u32,
    // Entries follow...
}

/// Memory map entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub typ: u32,
    pub reserved: u32,
}

impl MemoryMapEntry {
    pub fn memory_type(&self) -> MemoryType {
        MemoryType::from_u32(self.typ)
    }

    pub fn end_addr(&self) -> u64 {
        self.base_addr + self.length
    }
}

/// Memory region descriptor for kernel use
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: usize,
    pub end: usize,
    pub typ: MemoryType,
}

/// Parsed Multiboot2 boot information
pub struct ParsedBootInfo {
    /// Available RAM regions (up to 16)
    pub ram_regions: [Option<MemoryRegion>; 16],
    pub ram_count: usize,
    /// Total available RAM in bytes
    pub total_ram: usize,
    /// Highest usable physical address
    pub phys_max: usize,
}

impl ParsedBootInfo {
    pub const fn empty() -> Self {
        Self {
            ram_regions: [None; 16],
            ram_count: 0,
            total_ram: 0,
            phys_max: 0,
        }
    }
}

/// Parse multiboot2 boot information structure.
///
/// # Safety
/// The `info_addr` must point to a valid Multiboot2 boot information structure.
pub unsafe fn parse(info_addr: usize) -> ParsedBootInfo {
    // SAFETY: Caller guarantees info_addr points to valid Multiboot2 structure
    let boot_info = unsafe { &*(info_addr as *const BootInfo) };
    let mut result = ParsedBootInfo::empty();

    let mut offset = mem::size_of::<BootInfo>();
    let end = boot_info.total_size as usize;

    while offset < end {
        // Align to 8 bytes
        offset = (offset + 7) & !7;
        if offset >= end {
            break;
        }

        // SAFETY: We're within the boot info structure bounds
        let tag = unsafe { &*((info_addr + offset) as *const Tag) };

        if tag.typ == TagType::End as u32 {
            break;
        }

        if tag.typ == TagType::MemoryMap as u32 {
            // SAFETY: Tag is a valid memory map tag
            unsafe { parse_memory_map(info_addr + offset, &mut result) };
        }

        offset += tag.size as usize;
    }

    result
}

/// Parse memory map tag
unsafe fn parse_memory_map(tag_addr: usize, result: &mut ParsedBootInfo) {
    // SAFETY: Caller guarantees tag_addr points to valid memory map tag
    let mmap_tag = unsafe { &*(tag_addr as *const MemoryMapTag) };
    let entry_size = mmap_tag.entry_size as usize;
    let entries_start = tag_addr + mem::size_of::<MemoryMapTag>();
    let entries_end = tag_addr + mmap_tag.size as usize;

    let mut addr = entries_start;
    while addr < entries_end && result.ram_count < 16 {
        // SAFETY: We're within the memory map tag bounds
        let entry = unsafe { &*(addr as *const MemoryMapEntry) };

        let region = MemoryRegion {
            start: entry.base_addr as usize,
            end: entry.end_addr() as usize,
            typ: entry.memory_type(),
        };

        // Track all available RAM
        if region.typ.is_usable() {
            result.ram_regions[result.ram_count] = Some(region);
            result.ram_count += 1;
            result.total_ram += region.end - region.start;

            if region.end > result.phys_max {
                result.phys_max = region.end;
            }
        }

        addr += entry_size;
    }
}

/// Iterator over memory map entries
pub struct MemoryMapIter {
    current: usize,
    end: usize,
    entry_size: usize,
}

impl Iterator for MemoryMapIter {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        let entry = unsafe { &*(self.current as *const MemoryMapEntry) };
        self.current += self.entry_size;

        Some(MemoryRegion {
            start: entry.base_addr as usize,
            end: entry.end_addr() as usize,
            typ: entry.memory_type(),
        })
    }
}

/// Global storage for parsed boot info (set during early boot)
/// TEAM_267: Using atomic pointer for Rust 2024 compatibility
use core::sync::atomic::{AtomicPtr, Ordering};

static BOOT_INFO_PTR: AtomicPtr<ParsedBootInfo> = AtomicPtr::new(core::ptr::null_mut());

// Static storage for the boot info (initialized once)
static mut BOOT_INFO_STORAGE: ParsedBootInfo = ParsedBootInfo {
    ram_regions: [None; 16],
    ram_count: 0,
    total_ram: 0,
    phys_max: 0,
};

/// Initialize boot info from multiboot2 structure.
///
/// # Safety
/// Must be called exactly once during early boot with valid multiboot2 info address.
pub unsafe fn init(info_addr: usize) {
    let parsed = unsafe { parse(info_addr) };
    
    // Store in static storage
    unsafe {
        BOOT_INFO_STORAGE = parsed;
        BOOT_INFO_PTR.store(
            core::ptr::addr_of_mut!(BOOT_INFO_STORAGE),
            Ordering::Release,
        );
    }
}

/// Get reference to parsed boot info.
pub fn boot_info() -> Option<&'static ParsedBootInfo> {
    let ptr = BOOT_INFO_PTR.load(Ordering::Acquire);
    if ptr.is_null() {
        None
    } else {
        // SAFETY: Once set, BOOT_INFO_STORAGE is never modified again
        unsafe { Some(&*ptr) }
    }
}

/// Iterate over available RAM regions.
pub fn for_each_ram_region<F: FnMut(&MemoryRegion)>(mut f: F) {
    if let Some(info) = boot_info() {
        for region in info.ram_regions.iter().flatten() {
            f(region);
        }
    }
}

/// Get total available RAM in bytes.
pub fn total_ram() -> usize {
    boot_info().map(|i| i.total_ram).unwrap_or(0)
}

/// Get highest usable physical address.
pub fn phys_max() -> usize {
    boot_info().map(|i| i.phys_max).unwrap_or(0)
}

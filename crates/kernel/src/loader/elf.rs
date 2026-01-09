//! TEAM_073: ELF64 Parser and Loader for AArch64.
//!
//! Implements ELF binary parsing following the ELF64 specification.
//! Only supports statically-linked executable files (ET_EXEC).
//!
//! ## ELF Reference
//! - ELF64 Header: 64 bytes
//! - Program Header: 56 bytes each
//! - AArch64 Machine Type: EM_AARCH64 = 183

use crate::memory::user as mm_user;
use los_hal::mmu::{self, PAGE_SIZE, PageFlags};

/// ELF Magic Number: 0x7F 'E' 'L' 'F'
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// ELF Class: 64-bit
pub const ELFCLASS64: u8 = 2;

/// ELF Data Encoding: Little Endian
pub const ELFDATA2LSB: u8 = 1;

/// ELF Type: Executable
pub const ET_EXEC: u16 = 2;

/// TEAM_354: ELF Type: Shared object / PIE
pub const ET_DYN: u16 = 3;

/// ELF Machine: AArch64
pub const EM_AARCH64: u16 = 183;

/// Program Header Type: Loadable segment
pub const PT_LOAD: u32 = 1;

/// Segment Flags
pub const PF_X: u32 = 1; // Execute
#[allow(dead_code)]
pub const PF_W: u32 = 2; // Write
#[allow(dead_code)]
pub const PF_R: u32 = 4; // Read

use los_error::define_kernel_error;

define_kernel_error! {
    /// TEAM_073: ELF parsing errors.
    /// TEAM_152: Added error codes (0x02xx) per unified error system plan.
    /// TEAM_155: Migrated to define_kernel_error! macro.
    pub enum ElfError(0x02) {
        /// Data too short to contain header
        TooShort = 0x01 => "ELF data too short",
        /// Invalid ELF magic number
        InvalidMagic = 0x02 => "Invalid ELF magic number",
        /// Not a 64-bit ELF
        Not64Bit = 0x03 => "Not a 64-bit ELF",
        /// Not little-endian
        NotLittleEndian = 0x04 => "Not little-endian",
        /// Not an executable file
        NotExecutable = 0x05 => "Not an executable file",
        /// Not for AArch64
        WrongArchitecture = 0x06 => "Wrong architecture (not AArch64)",
        /// Invalid program header offset
        InvalidProgramHeader = 0x07 => "Invalid program header",
        /// Failed to allocate memory for segment
        AllocationFailed = 0x08 => "Memory allocation failed",
        /// Failed to map memory
        MappingFailed = 0x09 => "Memory mapping failed",
    }
}

/// TEAM_073: ELF64 File Header.
///
/// Offset and size from ELF64 specification.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Elf64Header {
    /// Magic number and identification
    pub e_ident: [u8; 16],
    /// Object file type
    pub e_type: u16,
    /// Machine type
    pub e_machine: u16,
    /// Object file version
    pub e_version: u32,
    /// Entry point address
    pub e_entry: u64,
    /// Program header offset
    pub e_phoff: u64,
    /// Section header offset
    pub e_shoff: u64,
    /// Processor-specific flags
    pub e_flags: u32,
    /// ELF header size
    pub e_ehsize: u16,
    /// Program header entry size
    pub e_phentsize: u16,
    /// Number of program headers
    pub e_phnum: u16,
    /// Section header entry size
    pub e_shentsize: u16,
    /// Number of section headers
    pub e_shnum: u16,
    /// Section name string table index
    pub e_shstrndx: u16,
}

impl Elf64Header {
    /// Size of the ELF64 header in bytes
    pub const SIZE: usize = 64;

    /// Parse an ELF64 header from raw bytes.
    pub fn parse(data: &[u8]) -> Result<Self, ElfError> {
        if data.len() < Self::SIZE {
            return Err(ElfError::TooShort);
        }

        use core::convert::TryInto;

        // Manually parse fields to avoid alignment/pointer issues
        let e_ident: [u8; 16] = data[0..16].try_into().unwrap(); // Infallible due to size check
        log::trace!("ELF Header e_ident verified");
        let e_type = u16::from_le_bytes(data[16..18].try_into().unwrap());
        let e_machine = u16::from_le_bytes(data[18..20].try_into().unwrap());
        let e_version = u32::from_le_bytes(data[20..24].try_into().unwrap());
        let e_entry = u64::from_le_bytes(data[24..32].try_into().unwrap());
        let e_phoff = u64::from_le_bytes(data[32..40].try_into().unwrap());
        let e_shoff = u64::from_le_bytes(data[40..48].try_into().unwrap());
        let e_flags = u32::from_le_bytes(data[48..52].try_into().unwrap());
        let e_ehsize = u16::from_le_bytes(data[52..54].try_into().unwrap());
        let e_phentsize = u16::from_le_bytes(data[54..56].try_into().unwrap());
        let e_phnum = u16::from_le_bytes(data[56..58].try_into().unwrap());
        let e_shentsize = u16::from_le_bytes(data[58..60].try_into().unwrap());
        let e_shnum = u16::from_le_bytes(data[60..62].try_into().unwrap());
        let e_shstrndx = u16::from_le_bytes(data[62..64].try_into().unwrap());

        let header = Elf64Header {
            e_ident,
            e_type,
            e_machine,
            e_version,
            e_entry,
            e_phoff,
            e_shoff,
            e_flags,
            e_ehsize,
            e_phentsize,
            e_phnum,
            e_shentsize,
            e_shnum,
            e_shstrndx,
        };

        // Validate magic
        if header.e_ident[0..4] != ELF_MAGIC {
            return Err(ElfError::InvalidMagic);
        }

        // Validate class (64-bit)
        if header.e_ident[4] != ELFCLASS64 {
            return Err(ElfError::Not64Bit);
        }

        // Validate endianness (little-endian)
        if header.e_ident[5] != ELFDATA2LSB {
            return Err(ElfError::NotLittleEndian);
        }

        // TEAM_354: Validate type (executable or PIE)
        if header.e_type != ET_EXEC && header.e_type != ET_DYN {
            return Err(ElfError::NotExecutable);
        }

        // Validate machine (Architecture-specific)
        if header.e_machine != crate::arch::ELF_MACHINE {
            return Err(ElfError::WrongArchitecture);
        }

        Ok(header)
    }

    /// Get the entry point address.
    pub fn entry_point(&self) -> usize {
        self.e_entry as usize
    }
}

/// TEAM_073: ELF64 Program Header.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Elf64ProgramHeader {
    /// Segment type
    pub p_type: u32,
    /// Segment flags
    pub p_flags: u32,
    /// Offset in file
    pub p_offset: u64,
    /// Virtual address in memory
    pub p_vaddr: u64,
    /// Physical address (unused)
    pub p_paddr: u64,
    /// Size in file
    pub p_filesz: u64,
    /// Size in memory
    pub p_memsz: u64,
    /// Segment alignment
    pub p_align: u64,
}

impl Elf64ProgramHeader {
    /// Size of a program header entry in bytes
    pub const SIZE: usize = 56;

    /// Check if this is a loadable segment
    pub fn is_loadable(&self) -> bool {
        self.p_type == PT_LOAD
    }

    /// Get the virtual address
    pub fn vaddr(&self) -> usize {
        self.p_vaddr as usize
    }

    /// Get the file offset
    pub fn offset(&self) -> usize {
        self.p_offset as usize
    }

    /// Get the size in file
    pub fn filesz(&self) -> usize {
        self.p_filesz as usize
    }

    /// Get the size in memory (includes .bss)
    pub fn memsz(&self) -> usize {
        self.p_memsz as usize
    }

    /// Get page flags based on segment flags
    pub fn page_flags(&self) -> PageFlags {
        let flags = self.p_flags;

        if (flags & PF_X != 0) && (flags & PF_W != 0) {
            // TEAM_308: Both writable and executable (common in single-segment ELFs)
            PageFlags::USER_CODE_DATA
        } else if flags & PF_X != 0 {
            // Executable segment (code)
            PageFlags::USER_CODE
        } else {
            // Data segment
            PageFlags::USER_DATA
        }
    }
}

/// TEAM_073: Parsed ELF file ready for loading.
pub struct Elf<'a> {
    /// Raw ELF data
    data: &'a [u8],
    /// Parsed header (owned copy to avoid alignment issues with packed structs)
    header: Elf64Header,
}

impl<'a> Elf<'a> {
    /// Parse an ELF file from raw bytes.
    pub fn parse(data: &'a [u8]) -> Result<Self, ElfError> {
        let header = Elf64Header::parse(data)?;
        Ok(Self { data, header })
    }

    /// Get the entry point address.
    pub fn entry_point(&self) -> usize {
        self.header.entry_point()
    }

    /// TEAM_217: Get the offset of the program headers.
    pub fn program_headers_offset(&self) -> usize {
        self.header.e_phoff as usize
    }

    /// TEAM_217: Get the number of program headers.
    pub fn program_headers_count(&self) -> usize {
        self.header.e_phnum as usize
    }

    /// TEAM_354: Check if this is a position-independent executable (PIE)
    pub fn is_pie(&self) -> bool {
        self.header.e_type == ET_DYN
    }

    /// TEAM_354: Get the base address where the ELF is loaded.
    /// - ET_EXEC: 0 (addresses are absolute)
    /// - ET_DYN: 0x10000 (fixed base for PIE, above null page)
    pub fn load_base(&self) -> usize {
        if self.is_pie() {
            0x10000  // 64KB, above null page
        } else {
            0  // ET_EXEC uses absolute addresses
        }
    }

    /// Iterate over program headers.
    pub fn program_headers(&self) -> impl Iterator<Item = Elf64ProgramHeader> + 'a {
        let phoff = self.header.e_phoff as usize;
        let phnum = self.header.e_phnum as usize;
        let phentsize = self.header.e_phentsize as usize;
        let data = self.data; // Capture slice

        (0..phnum).filter_map(move |i| {
            let offset = phoff + i * phentsize;
            if offset + Elf64ProgramHeader::SIZE <= data.len() {
                // Read using read_unaligned for safety
                // SAFETY: The offset and size are validated against the data length.
                // We use read_unaligned because the program header might not be aligned.
                let phdr = unsafe { 
                    core::ptr::read_unaligned(data.as_ptr().add(offset) as *const Elf64ProgramHeader) 
                };
                Some(phdr)
            } else {
                None
            }
        })
    }

    /// Load the ELF into a user address space.
    ///
    /// # Arguments
    /// * `ttbr0_phys` - Physical address of user L0 page table
    ///
    /// # Returns
    /// Tuple of (entry_point, initial_brk) on success.
    pub fn load(&self, ttbr0_phys: usize) -> Result<(usize, usize), ElfError> {
        let mut max_vaddr = 0;
        // TEAM_354: Get load base for PIE support
        let load_base = self.load_base();

        // TEAM_297 BREADCRUMB: DEAD_END - Missing GOT/PLT Relocations.
        // Userspace binaries are statically linked (ET_EXEC).
        // GOT is pre-populated by linker. No runtime relocations are needed or performed.
        // If GOT is wrong, it's due to memory corruption (Hypothesis 7), not missing relocations.
        for phdr in self.program_headers() {
            if !phdr.is_loadable() {
                continue;
            }

            // TEAM_354: Apply load_base offset to vaddr for PIE binaries
            let vaddr = load_base + phdr.vaddr();
            let memsz = phdr.memsz();
            let filesz = phdr.filesz();
            let offset = phdr.offset();
            let flags = phdr.page_flags();

            // Track highest address for brk
            let segment_end = vaddr + memsz;
            if segment_end > max_vaddr {
                max_vaddr = segment_end;
            }

            // Allocate pages for this segment
            let page_start = vaddr & !0xFFF;
            let page_end = (vaddr + memsz + 0xFFF) & !0xFFF;
            let num_pages = (page_end - page_start) / PAGE_SIZE;

            // Allocate physical pages
            for i in 0..num_pages {
                let page_va = page_start + i * PAGE_SIZE;

                // TEAM_079: Check if page is already mapped (segments can share pages)
                // If already mapped, skip - keeps first mapping's flags (typically executable)
                let l0_va = mmu::phys_to_virt(ttbr0_phys);
                // SAFETY: ttbr0_phys is a valid page table root. We are walking it
                // to check for existing mappings.
                let already_mapped = mmu::walk_to_entry(
                    unsafe { &mut *(l0_va as *mut mmu::PageTable) },
                    page_va,
                    3,
                    false,
                )
                .map(|w| w.table.entry(w.index).is_valid())
                .unwrap_or(false);

                if already_mapped {
                    // TEAM_212: Page already mapped - check if we need to upgrade permissions
                    // If new segment needs RW (USER_DATA) but page was mapped RO (USER_CODE),
                    // we need to upgrade to USER_CODE_DATA (RWX) to allow both execute and write
                    // TEAM_297 BREADCRUMB: INVESTIGATING - Shared page mapping.
                    // Crash happens in function called via GOT, and GOT/rodata share a page.
                    // Verifying if permissions are correctly upgraded and data is preserved.
                    if flags == PageFlags::USER_DATA {
                        let l0_va = mmu::phys_to_virt(ttbr0_phys);
                        // SAFETY: ttbr0_phys is a valid page table root.
                        if let Ok(walk) = mmu::walk_to_entry(
                            unsafe { &mut *(l0_va as *mut mmu::PageTable) },
                            page_va,
                            3,
                            false,
                        ) {
                            let entry = walk.table.entry_mut(walk.index);
                            let phys = entry.address();
                            // Use USER_CODE_DATA (RWX) to preserve execute + add write
                            entry.set(phys, PageFlags::USER_CODE_DATA | PageFlags::TABLE);
                            mmu::tlb_flush_page(page_va);
                        }
                    }
                    continue;
                }

                // Allocate a physical page
                let phys = crate::memory::FRAME_ALLOCATOR
                    .alloc_page()
                    .ok_or(ElfError::AllocationFailed)?;

                // Zero the page first
                // SAFETY: phys is a newly allocated page. We zero it to avoid
                // leaking data to userspace.
                unsafe {
                    let page_ptr = mmu::phys_to_virt(phys) as *mut u8;
                    core::ptr::write_bytes(page_ptr, 0, PAGE_SIZE);
                }

                // Map into user space
                // SAFETY: The page and addresses are validated during ELF loading.
                unsafe {
                    mm_user::map_user_page(ttbr0_phys, page_va, phys, flags)
                        .map_err(|_| ElfError::MappingFailed)?;
                }
            }

            // Copy segment data from ELF file
            if filesz > 0 {
                let src = &self.data[offset..offset + filesz];

                // Calculate destination in physical memory
                // We need to write to the physical pages we just mapped
                for (i, byte) in src.iter().enumerate() {
                    let dst_va = vaddr + i;
                    let page_va = dst_va & !0xFFF;
                    let page_offset = dst_va & 0xFFF;

                    // DEBUG: Print first byte copy attempt
                    if i == 0 {
                        log::trace!(
                            "[ELF] Copying segment: src[0]={:x} to VA {:x}",
                            *byte,
                            dst_va
                        );
                    }

                    // Get the L0 table to find the physical address
                    let l0_va = mmu::phys_to_virt(ttbr0_phys);

                    // Walk page tables to find physical address
                    // SAFETY: ttbr0_phys is a valid page table root.
                    if let Ok(walk) = mmu::walk_to_entry(
                        unsafe { &mut *(l0_va as *mut mmu::PageTable) },
                        page_va,
                        3,
                        false,
                    ) {
                        let entry_phys = walk.table.entry(walk.index).address();
                        let dst_phys = entry_phys + page_offset;
                        let dst = mmu::phys_to_virt(dst_phys) as *mut u8;

                        if i == 0 {
                            log::trace!(
                                "[ELF] Resolved PA [MASKED] -> Kernel VA [MASKED]"
                            );
                        }

                            /*
                            if dst_va == 0x11fb8 {
                                log::warn!("[ELF] WRITING GOT ENTRY at {:x}: val={:x}", dst_va, *byte);
                            }
                            */
                            // SAFETY: dst is a valid pointer within a mapped page
                            // in the user address space.
                            unsafe {
                                *dst = *byte;
                            }
                    }
                }
            }

            // .bss is already zeroed (we zeroed pages above)
        }

        // Calculate initial brk (page-aligned end of loaded segments)
        let initial_brk = (max_vaddr + 0xFFF) & !0xFFF;

        // TEAM_354: Process relocations for PIE binaries
        if self.is_pie() {
            let reloc_count = self.process_relocations(ttbr0_phys, load_base)?;
            log::debug!("[ELF] Applied {} relocations for PIE binary", reloc_count);
        }

        // TEAM_354: Return adjusted entry point
        let entry_point = load_base + self.header.e_entry as usize;
        Ok((entry_point, initial_brk))
    }

    /// TEAM_354: Process relocations for PIE binaries using goblin
    fn process_relocations(&self, ttbr0_phys: usize, load_base: usize) -> Result<usize, ElfError> {
        use goblin::elf::Elf as GoblinElf;
        
        // Parse with goblin to get relocations
        let elf = match GoblinElf::parse(self.data) {
            Ok(e) => e,
            Err(_) => {
                log::warn!("[ELF] Goblin parse failed for relocations");
                return Ok(0);
            }
        };

        let mut count = 0;

        // Process .rela.dyn relocations (goblin parses these automatically)
        for rela in &elf.dynrelas {
            let addend = rela.r_addend.unwrap_or(0);
            self.apply_relocation(ttbr0_phys, load_base, rela.r_offset as usize, addend, rela.r_type)?;
            count += 1;
        }

        Ok(count)
    }

    /// TEAM_354: Apply a single relocation
    fn apply_relocation(
        &self,
        ttbr0_phys: usize,
        load_base: usize,
        r_offset: usize,
        r_addend: i64,
        r_type: u32,
    ) -> Result<(), ElfError> {
        // Architecture-specific relocation constants
        #[cfg(target_arch = "aarch64")]
        const R_RELATIVE: u32 = 1027; // R_AARCH64_RELATIVE

        #[cfg(target_arch = "x86_64")]
        const R_RELATIVE: u32 = 8; // R_X86_64_RELATIVE

        match r_type {
            R_RELATIVE => {
                // R_*_RELATIVE: *target = load_base + addend
                let target_va = load_base + r_offset;
                let value = (load_base as i64 + r_addend) as u64;
                self.write_user_u64(ttbr0_phys, target_va, value)?;
            }
            other => {
                log::trace!("[ELF] Skipping unsupported reloc type {}", other);
            }
        }

        Ok(())
    }

    /// TEAM_354: Write a u64 to user address space
    fn write_user_u64(&self, ttbr0_phys: usize, va: usize, value: u64) -> Result<(), ElfError> {
        let l0_va = mmu::phys_to_virt(ttbr0_phys);
        let l0 = unsafe { &mut *(l0_va as *mut mmu::PageTable) };

        let page_va = va & !0xFFF;
        let page_offset = va & 0xFFF;

        match mmu::walk_to_entry(l0, page_va, 3, false) {
            Ok(walk) => {
                let phys = walk.table.entry(walk.index).address() + page_offset;
                let ptr = mmu::phys_to_virt(phys) as *mut u64;
                unsafe { ptr.write_unaligned(value); }
                Ok(())
            }
            Err(_) => Err(ElfError::MappingFailed),
        }
    }
}

use los_hal::traits::PageAllocator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_magic() {
        assert_eq!(ELF_MAGIC, [0x7F, b'E', b'L', b'F']);
    }

    #[test]
    fn test_header_size() {
        assert_eq!(Elf64Header::SIZE, 64);
        assert_eq!(core::mem::size_of::<Elf64Header>(), 64);
    }

    #[test]
    fn test_program_header_size() {
        assert_eq!(Elf64ProgramHeader::SIZE, 56);
        assert_eq!(core::mem::size_of::<Elf64ProgramHeader>(), 56);
    }

    #[test]
    fn test_parse_invalid() {
        // Too short
        assert_eq!(
            Elf64Header::parse(&[0; 10]).unwrap_err(),
            ElfError::TooShort
        );

        // Invalid magic
        let mut data = [0u8; 64];
        assert_eq!(
            Elf64Header::parse(&data).unwrap_err(),
            ElfError::InvalidMagic
        );

        // Valid magic but wrong class
        data[0] = 0x7F;
        data[1] = b'E';
        data[2] = b'L';
        data[3] = b'F';
        data[4] = 1; // 32-bit
        assert_eq!(Elf64Header::parse(&data).unwrap_err(), ElfError::Not64Bit);
    }
}

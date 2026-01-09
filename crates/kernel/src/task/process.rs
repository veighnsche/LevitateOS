//! TEAM_073: User process spawning and management.
//!
//! This module provides the high-level interface for creating and
//! running user processes.
//!
//! TEAM_158: Behavior IDs [PROC1]-[PROC4] for traceability.

use crate::loader::elf::Elf;
use crate::loader::elf::ElfError;
use crate::memory::user as mm_user;
use crate::task::fd_table::SharedFdTable;
use crate::task::user::UserTask;
use los_hal::mmu::MmuError;

use los_error::define_kernel_error;

define_kernel_error! {
    /// TEAM_073: Error type for process spawning.
    /// TEAM_155: Migrated to define_kernel_error! macro.
    pub enum SpawnError(0x03) {
        /// ELF parsing/loading failed
        Elf(ElfError) = 0x01 => "ELF loading failed",
        /// Page table creation failed
        PageTable(MmuError) = 0x02 => "Page table creation failed",
        /// Stack setup failed
        Stack(MmuError) = 0x03 => "Stack setup failed",
    }
}

impl From<ElfError> for SpawnError {
    fn from(e: ElfError) -> Self {
        SpawnError::Elf(e) // TEAM_152: Now preserves context
    }
}

/// [PROC1] Spawn a user process from an ELF binary in memory.
/// [PROC2] Creates user page table for the new process.
///
/// # Arguments
/// * `elf_data` - Raw ELF file contents
/// * `fd_table` - File descriptor table to use
///
/// # Returns
/// A `UserTask` ready to be scheduled, or an error.
pub fn spawn_from_elf(elf_data: &[u8], fd_table: SharedFdTable) -> Result<UserTask, SpawnError> {
    // TEAM_169: Delegate to spawn_from_elf_with_args with empty args
    spawn_from_elf_with_args(elf_data, &[], &[], fd_table)
}

/// TEAM_169: Spawn a user process with arguments.
///
/// Per Phase 2 Q5 decision: Stack-based argument passing (Linux ABI compatible).
///
/// # Arguments
/// * `elf_data` - Raw ELF file contents
/// * `args` - Command line arguments (argv)
/// * `envs` - Environment variables (envp)
/// * `fd_table` - File descriptor table to use
///
/// # Returns
/// A `UserTask` ready to be scheduled, or an error.
pub fn spawn_from_elf_with_args(
    elf_data: &[u8],
    args: &[&str],
    envs: &[&str],
    fd_table: SharedFdTable,
) -> Result<UserTask, SpawnError> {
    log::debug!("[SPAWN] Parsing ELF header ({} bytes)...", elf_data.len());
    // 1. Parse ELF
    let elf = Elf::parse(elf_data)?;
    log::debug!("[SPAWN] ELF parsed.");

    // [PROC2] Create user page table
    log::debug!("[SPAWN] Creating user page table...");
    let ttbr0_phys = mm_user::create_user_page_table() // [PROC2]
        .ok_or(SpawnError::PageTable(MmuError::AllocationFailed))?;

    // 3. Load ELF segments into user address space
    log::debug!("[SPAWN] Loading segments...");
    let (entry_point, brk) = elf.load(ttbr0_phys)?;

    // 4. Set up user stack
    log::debug!("[SPAWN] Setting up stack...");
    let stack_pages = mm_user::layout::STACK_SIZE / los_hal::mmu::PAGE_SIZE;
    let stack_top =
        unsafe { mm_user::setup_user_stack(ttbr0_phys, stack_pages).map_err(SpawnError::Stack)? };

    // TEAM_216: Always set up argc/argv/envp on stack.
    // Even if empty, this ensures SP is moved into a mapped page (Rule 4).
    log::debug!("[SPAWN] Setting up stack arguments...");

    // TEAM_217: Collect ELF info for auxv
    let mut auxv = alloc::vec::Vec::new();
    auxv.push(crate::memory::user::AuxEntry {
        a_type: crate::memory::user::AT_PHDR,
        a_val: elf.program_headers_offset() + elf.load_base(),
    });
    auxv.push(crate::memory::user::AuxEntry {
        a_type: crate::memory::user::AT_PHENT,
        a_val: crate::loader::elf::Elf64ProgramHeader::SIZE,
    });
    auxv.push(crate::memory::user::AuxEntry {
        a_type: crate::memory::user::AT_PHNUM,
        a_val: elf.program_headers_count(),
    });

    let user_sp = mm_user::setup_stack_args(ttbr0_phys, stack_top, args, envs, &auxv)
        .map_err(SpawnError::Stack)?;

    // 5. Create UserTask
    let task = UserTask::new(entry_point, user_sp, ttbr0_phys, brk, fd_table);

    log::debug!(
        "[SPAWN] Success: PID={} entry=0x{:x} sp=0x{:x}",
        task.pid.0,
        entry_point,
        user_sp
    );

    Ok(task)
}

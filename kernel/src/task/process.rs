//! TEAM_073: User process spawning and management.
//!
//! This module provides the high-level interface for creating and
//! running user processes.

use crate::loader::elf::Elf;
use crate::loader::elf::ElfError;
use crate::task::user::UserTask;
use crate::task::user_mm;

/// TEAM_073: Error type for process spawning.
#[derive(Debug)]
pub enum SpawnError {
    /// Failed to parse ELF
    ElfError,
    /// Failed to create page table
    PageTableCreation,
    /// Failed to set up stack
    StackSetup,
}

impl From<ElfError> for SpawnError {
    fn from(_e: ElfError) -> Self {
        SpawnError::ElfError
    }
}

/// TEAM_073: Spawn a user process from an ELF binary in memory.
///
/// # Arguments
/// * `elf_data` - Raw ELF file contents
///
/// # Returns
/// A `UserTask` ready to be scheduled, or an error.
pub fn spawn_from_elf(elf_data: &[u8]) -> Result<UserTask, SpawnError> {
    levitate_hal::println!("[SPAWN] Parsing ELF header ({} bytes)...", elf_data.len());
    // 1. Parse ELF
    let elf = Elf::parse(elf_data)?;
    levitate_hal::println!("[SPAWN] ELF parsed.");

    // 2. Create user page table
    levitate_hal::println!("[SPAWN] Creating user page table...");
    let ttbr0_phys = user_mm::create_user_page_table().ok_or(SpawnError::PageTableCreation)?;

    // 3. Load ELF segments into user address space
    levitate_hal::println!("[SPAWN] Loading segments...");
    let (entry_point, brk) = elf.load(ttbr0_phys)?;

    // 4. Set up user stack
    levitate_hal::println!("[SPAWN] Setting up stack...");
    let stack_pages = user_mm::layout::STACK_SIZE / levitate_hal::mmu::PAGE_SIZE;
    let user_sp = unsafe {
        user_mm::setup_user_stack(ttbr0_phys, stack_pages).map_err(|_| SpawnError::StackSetup)?
    };

    // 5. Create UserTask
    let task = UserTask::new(entry_point, user_sp, ttbr0_phys, brk);

    levitate_hal::println!(
        "[SPAWN] Success: PID={} entry=0x{:x} sp=0x{:x}",
        task.pid.0,
        entry_point,
        user_sp
    );

    Ok(task)
}

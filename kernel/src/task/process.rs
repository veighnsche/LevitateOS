//! TEAM_073: User process spawning and management.
//!
//! This module provides the high-level interface for creating and
//! running user processes.

use crate::loader::elf::{Elf, ElfError};
use crate::task::user::{Pid, ProcessState, UserTask};
use crate::task::user_mm;
use levitate_hal::mmu::PageFlags;

/// TEAM_073: Error type for process spawning.
#[derive(Debug)]
pub enum SpawnError {
    /// Failed to parse ELF
    ElfError(ElfError),
    /// Failed to create page table
    PageTableCreation,
    /// Failed to set up stack
    StackSetup,
    /// File not found
    NotFound,
}

impl From<ElfError> for SpawnError {
    fn from(e: ElfError) -> Self {
        SpawnError::ElfError(e)
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
    levitate_hal::println!("[SPAWN] Parsing ELF header... data len {}", elf_data.len());
    // 1. Parse ELF
    let elf = Elf::parse(elf_data)?;

    // 2. Create user page table
    let ttbr0_phys = user_mm::create_user_page_table().ok_or(SpawnError::PageTableCreation)?;

    // 3. Load ELF segments into user address space
    let (entry_point, brk) = elf.load(ttbr0_phys)?;

    // 4. Set up user stack
    let stack_pages = user_mm::layout::STACK_SIZE / levitate_hal::mmu::PAGE_SIZE;
    let user_sp = unsafe {
        user_mm::setup_user_stack(ttbr0_phys, stack_pages).map_err(|_| SpawnError::StackSetup)?
    };

    // 5. Create UserTask
    let task = UserTask::new(entry_point, user_sp, ttbr0_phys, brk);

    levitate_hal::println!(
        "[SPAWN] Created user process PID={} entry=0x{:x} sp=0x{:x}",
        task.pid.0,
        entry_point,
        user_sp
    );

    Ok(task)
}

/// TEAM_073: Start executing a user task.
///
/// This function does not return - it enters user mode.
///
/// # Safety
/// - The task's page table must be valid and mapped
/// - TTBR0 will be switched to the task's page table
pub unsafe fn enter_user_task(task: &UserTask) -> ! {
    // 1. Switch TTBR0 to user page table
    unsafe {
        levitate_hal::mmu::switch_ttbr0(task.ttbr0);
    }

    // 2. Enter user mode
    unsafe { crate::task::user::enter_user_mode(task.entry_point, task.user_sp) }
}

/// TEAM_073: Run a user program from the initramfs.
///
/// This is a convenience function that:
/// 1. Finds the file in initramfs
/// 2. Spawns a process
/// 3. Enters user mode
///
/// # Arguments
/// * `path` - Path to the file in initramfs (without leading /)
/// * `initramfs` - The CPIO archive
pub fn run_from_initramfs(path: &str, initramfs: &crate::fs::initramfs::CpioArchive) -> ! {
    levitate_hal::println!("[SPAWN] Looking for '{}' in initramfs...", path);

    // Find the file
    for entry in initramfs.iter() {
        if entry.name == path {
            levitate_hal::println!("[SPAWN] Found '{}' ({} bytes)", path, entry.data.len());

            // Spawn process
            match spawn_from_elf(entry.data) {
                Ok(task) => {
                    levitate_hal::println!("[SPAWN] Starting user process...");
                    unsafe {
                        enter_user_task(&task);
                    }
                }
                Err(e) => {
                    levitate_hal::println!("[SPAWN] Failed to spawn: {:?}", e);
                    panic!("Failed to spawn user process");
                }
            }
        }
    }

    levitate_hal::println!("[SPAWN] File '{}' not found in initramfs", path);
    panic!("User program not found");
}

//! TEAM_188: Process table for parent-child tracking and waitpid.
//!
//! This module maintains a global table of all processes, tracking:
//! - Parent-child relationships
//! - Exit codes for zombie processes
//! - Tasks waiting for specific PIDs to exit

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use los_hal::IrqSafeLock;

use crate::task::TaskControlBlock;

/// TEAM_188: Process ID type alias
pub type Pid = usize;

/// TEAM_188: Entry in the process table
pub struct ProcessEntry {
    /// The task control block (None if zombie — task exited but not reaped)
    pub task: Option<Arc<TaskControlBlock>>,
    /// Parent process ID (0 for init/orphans)
    /// TEAM_191: Reserved for waitpid(-1) - wait for any child process
    #[allow(dead_code)]
    pub parent_pid: Pid,
    /// Exit code (Some after exit, None while running)
    pub exit_code: Option<i32>,
    /// Tasks waiting for this process to exit
    pub waiters: Vec<Arc<TaskControlBlock>>,
}

/// TEAM_188: Global process table
/// Uses BTreeMap for deterministic iteration order (Rule 18: Determinism)
pub static PROCESS_TABLE: IrqSafeLock<BTreeMap<Pid, ProcessEntry>> =
    IrqSafeLock::new(BTreeMap::new());

/// TEAM_188: Register a new process in the table.
///
/// Called by sys_spawn_args when a new process is created.
///
/// # Arguments
/// * `pid` - The new process's PID
/// * `parent_pid` - The parent process's PID
/// * `task` - Arc to the task control block
pub fn register_process(pid: Pid, parent_pid: Pid, task: Arc<TaskControlBlock>) {
    let mut table = PROCESS_TABLE.lock();
    table.insert(
        pid,
        ProcessEntry {
            task: Some(task),
            parent_pid,
            exit_code: None,
            waiters: Vec::new(),
        },
    );
}

/// TEAM_188: Mark a process as exited and return waiters to wake.
///
/// Called by sys_exit when a process terminates.
///
/// # Arguments
/// * `pid` - The exiting process's PID
/// * `exit_code` - The exit code passed to sys_exit
///
/// # Returns
/// List of tasks that were waiting for this process (to be woken up)
pub fn mark_exited(pid: Pid, exit_code: i32) -> Vec<Arc<TaskControlBlock>> {
    let mut table = PROCESS_TABLE.lock();
    if let Some(entry) = table.get_mut(&pid) {
        entry.exit_code = Some(exit_code);
        entry.task = None; // Become zombie
        return entry.waiters.drain(..).collect();
    }
    Vec::new()
}

/// TEAM_188: Try to get exit code for a process.
///
/// Returns Some(exit_code) if the process has exited, None if still running.
///
/// # Arguments
/// * `pid` - The PID to check
pub fn try_wait(pid: Pid) -> Option<i32> {
    let table = PROCESS_TABLE.lock();
    table.get(&pid).and_then(|e| e.exit_code)
}

/// TEAM_188: Check if a process exists in the table.
/// TEAM_191: Reserved for waitpid validation and future process queries
#[allow(dead_code)]
pub fn process_exists(pid: Pid) -> bool {
    let table = PROCESS_TABLE.lock();
    table.contains_key(&pid)
}

/// TEAM_188: Add a waiter to a process.
///
/// Called by sys_waitpid when a parent needs to block waiting for a child.
///
/// # Arguments
/// * `pid` - The PID to wait for
/// * `waiter` - The task that is waiting
///
/// # Returns
/// * `Ok(())` - Waiter added successfully
/// * `Err(AlreadyExited)` - Process already exited
/// * `Err(NotFound)` - Process not in table
pub fn add_waiter(pid: Pid, waiter: Arc<TaskControlBlock>) -> Result<(), WaitError> {
    let mut table = PROCESS_TABLE.lock();
    if let Some(entry) = table.get_mut(&pid) {
        if entry.exit_code.is_some() {
            return Err(WaitError::AlreadyExited);
        }
        entry.waiters.push(waiter);
        return Ok(());
    }
    Err(WaitError::NotFound)
}

/// TEAM_188: Error type for add_waiter
#[derive(Debug)]
pub enum WaitError {
    /// Process already exited (check exit code instead)
    AlreadyExited,
    /// Process not found in table
    NotFound,
}

/// TEAM_188: Reap a zombie process (remove from table).
///
/// Called after waitpid successfully retrieves exit code.
///
/// # Arguments
/// * `pid` - The zombie PID to reap
pub fn reap_zombie(pid: Pid) {
    let mut table = PROCESS_TABLE.lock();
    // Only remove if it's actually a zombie (has exit code)
    if let Some(entry) = table.get(&pid) {
        if entry.exit_code.is_some() {
            table.remove(&pid);
        }
    }
}

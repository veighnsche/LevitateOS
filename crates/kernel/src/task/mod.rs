//! TEAM_158: Behavior IDs [MT1]-[MT17] for multitasking traceability.

pub mod fd_table; // TEAM_168: File descriptor table (Phase 10)
pub mod process;
pub mod process_table; // TEAM_188: Process table for waitpid
pub mod scheduler;
pub mod thread; // TEAM_230: Thread creation for sys_clone
pub mod user; // TEAM_073: Userspace support (Phase 8)
// TEAM_208: user_mm moved to crate::memory::user

extern crate alloc;
use crate::arch::{Context, cpu_switch_to};

use alloc::boxed::Box;
use alloc::string::String;
use core::sync::atomic::{AtomicU8, AtomicU32, AtomicUsize, Ordering};

/// TEAM_070: Hook called immediately after a context switch.
/// Used to release scheduler locks or perform cleanup.
#[unsafe(no_mangle)]
pub extern "C" fn post_switch_hook() {
    // TEAM_070: Prerequisite for Step 4 scheduler lock.
}

/// TEAM_071: Final exit handler for a task.
/// Marks the current task as Exited and yields to the scheduler.
/// The task will not be re-added to the ready queue.
/// TEAM_230: Added CLONE_CHILD_CLEARTID handling for thread join.
#[unsafe(no_mangle)]
pub extern "C" fn task_exit() -> ! {
    let task = current_task();

    // TEAM_230: Handle CLONE_CHILD_CLEARTID - clear tid and wake futex
    let clear_tid = task.clear_child_tid.load(Ordering::Acquire);
    if clear_tid != 0 {
        // SAFETY: user_va_to_kernel_ptr verified the address is mapped
        // and belongs to this task's address space (shared via CLONE_VM).
        if let Some(ptr) = crate::memory::user::user_va_to_kernel_ptr(task.ttbr0, clear_tid) {
            unsafe {
                *(ptr as *mut i32) = 0;
            }
        }
        // Wake one waiter on futex(clear_tid) for thread join
        crate::syscall::sync::futex_wake(clear_tid, 1);
    }

    // TEAM_071: Mark task as exited (Design Q2)
    task.set_state(TaskState::Exited);

    // Yield to next task without re-adding self to ready queue
    scheduler::SCHEDULER.schedule();

    // If we return here, no other tasks are ready - enter idle
    loop {
        crate::arch::cpu::wait_for_interrupt();
    }
}

/// TEAM_220: Terminate the current task as a result of a fatal signal.
/// Marks the process as exited in the process table and wakes waiters.
pub fn terminate_with_signal(sig: i32) -> ! {
    let task = current_task();
    let pid = task.id.0;

    // Record exit code as 128 + sig (Unix convention)
    let exit_code = 128 + sig;
    let waiters = process_table::mark_exited(pid, exit_code);

    // Wake up parent if waiting
    for waiter in waiters {
        waiter.set_state(TaskState::Ready);
        scheduler::SCHEDULER.add_task(waiter);
    }

    // Now terminate for real
    task_exit();
}

use alloc::sync::Arc;
use los_hal::IrqSafeLock;

/// TEAM_070: Pointer to the currently running task.
static CURRENT_TASK: IrqSafeLock<Option<Arc<TaskControlBlock>>> = IrqSafeLock::new(None);

/// TEAM_070: Get the currently running task as an Arc.
pub fn current_task() -> Arc<TaskControlBlock> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let pcr = crate::arch::cpu::get_pcr();
        let ptr = pcr.current_task_ptr as *const TaskControlBlock;
        if !ptr.is_null() {
            // TEAM_299: We know the Arc is held by the scheduler or CURRENT_TASK global.
            // Create a new Arc and increment its reference count.
            let arc = Arc::from_raw(ptr);
            let cloned = arc.clone();
            let _ = Arc::into_raw(arc); // Don't drop the original
            return cloned;
        }
    }

    CURRENT_TASK
        .lock()
        .as_ref()
        .cloned()
        .expect("current_task() called before scheduler init")
}

/// TEAM_070: Internal helper to set the current task.
pub unsafe fn set_current_task(task: Arc<TaskControlBlock>) {
    #[cfg(target_arch = "x86_64")]
    {
        let pcr = crate::arch::cpu::get_pcr();
        pcr.current_task_ptr = Arc::as_ptr(&task) as usize;
    }
    *CURRENT_TASK.lock() = Some(task);
}

/// [MT3] switch_to() updates CURRENT_TASK before switch.
/// [MT4] switch_to() no-ops when switching to same task.
pub fn switch_to(new_task: Arc<TaskControlBlock>) {
    let old_task = current_task();
    if Arc::ptr_eq(&old_task, &new_task) {
        return; // [MT4] no-op for same task
    }

    unsafe {
        // SAFETY: We cast to mut pointers for the assembly.
        // During switch, interrupts are disabled (usually) so this is safe.
        let old_ctx = &old_task.context as *const Context as *mut Context;
        let new_ctx = &new_task.context as *const Context;

        // TEAM_127: Fix Race Condition - Interrupts MUST be disabled during context switch
        // to prevent recursive scheduling or state corruption.
        let flags = los_hal::interrupts::disable();

        // [MT3] Update current task pointer before switch
        set_current_task(new_task.clone()); // TEAM_299: Clone Arc for set_current_task

        // TEAM_299: Switch Page Tables (CR3/TTBR0)
        // Critical for process isolation. Without this, new task runs in old task's address space.
        if new_task.ttbr0 != 0 {
            crate::arch::switch_mmu_config(new_task.ttbr0);
        }

        cpu_switch_to(old_ctx, new_ctx);

        los_hal::interrupts::restore(flags);
    }
}

/// [MT5] yield_now() re-adds current task to ready queue.
/// TEAM_208: Only re-add if task is not Blocked or Exited.
pub fn yield_now() {
    let task = current_task();
    let state = task.get_state();

    // TEAM_208: If task is blocked (e.g., waiting on futex), don't re-add to ready queue
    if state == TaskState::Blocked || state == TaskState::Exited {
        // Just schedule next task without re-adding current
        scheduler::SCHEDULER.schedule();
    } else {
        // TEAM_143: Single lock acquisition instead of add_task + schedule
        if let Some(next) = scheduler::SCHEDULER.yield_and_reschedule(task) {
            switch_to(next);
        }
    }
}

/// TEAM_070: Unique identifier for a task.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub usize);

impl TaskId {}

/// TEAM_070: Possible states of a task in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TaskState {
    #[allow(dead_code)]
    Ready = 0,
    Running = 1,
    #[allow(dead_code)]
    Blocked = 2,
    Exited = 3,
}

/// TEAM_070: Default stack size for kernel tasks (64KB).
#[allow(dead_code)]
pub const DEFAULT_STACK_SIZE: usize = 65536;

/// TEAM_070: Task Control Block (TCB).
/// Stores all information about a task.
/// TEAM_071: State uses AtomicU8 for safe mutation from task_exit.
#[repr(C)]
pub struct TaskControlBlock {
    #[allow(dead_code)]
    pub id: TaskId,
    /// TEAM_071: Atomic state for safe mutation without &mut self.
    state: AtomicU8,
    pub context: Context,
    /// The kernel stack for this task.
    /// None for the bootstrap task (which uses the boot stack).
    #[allow(dead_code)]
    stack: Option<Box<[u64]>>,
    #[allow(dead_code)]
    pub stack_top: usize,
    #[allow(dead_code)]
    pub stack_size: usize,
    /// Physical address of the task's L0 page table.
    #[allow(dead_code)]
    pub ttbr0: usize,
    /// User stack pointer (SP_EL0)
    pub user_sp: usize,
    /// User entry point
    pub user_entry: usize,
    /// TEAM_166: Process heap state for sbrk syscall
    pub heap: IrqSafeLock<ProcessHeap>,
    /// TEAM_168: File descriptor table
    pub fd_table: fd_table::SharedFdTable,
    /// TEAM_192: Current working directory
    pub cwd: IrqSafeLock<String>,
    /// TEAM_216: Pending signals bitmask
    pub pending_signals: AtomicU32,
    /// TEAM_216: Blocked signals bitmask
    pub blocked_signals: AtomicU32,
    /// TEAM_216: Signal handlers (userspace addresses)
    pub signal_handlers: IrqSafeLock<[usize; 32]>,
    /// TEAM_216: Signal trampoline address (in userspace)
    pub signal_trampoline: AtomicUsize,
    /// TEAM_228: Address to clear and wake on thread exit (for CLONE_CHILD_CLEARTID)
    pub clear_child_tid: AtomicUsize,
    /// TEAM_238: Virtual memory area tracking for munmap support
    pub vmas: IrqSafeLock<crate::memory::vma::VmaList>,
}

/// TEAM_220: Global tracking of the foreground process for shell control.
pub static FOREGROUND_PID: IrqSafeLock<usize> = IrqSafeLock::new(0);

impl TaskControlBlock {
    /// Set the state of the task.
    pub fn set_state(&self, state: TaskState) {
        self.state.store(state as u8, Ordering::Release);
    }

    /// TEAM_208: Get the current state of the task.
    pub fn get_state(&self) -> TaskState {
        match self.state.load(Ordering::Acquire) {
            0 => TaskState::Ready,
            1 => TaskState::Running,
            2 => TaskState::Blocked,
            3 => TaskState::Exited,
            _ => TaskState::Ready, // Fallback
        }
    }
}

impl TaskControlBlock {
    /// Create a TCB for the current (bootstrap) task.
    /// TEAM_316: Minimal version to avoid crash at 0x800200188
    pub fn new_bootstrap() -> Self {
        // TEAM_316: Use Default trait to let compiler handle initialization
        Self::default()
    }
}

impl Default for TaskControlBlock {
    fn default() -> Self {
        Self {
            id: TaskId(0),
            state: AtomicU8::new(TaskState::Running as u8),
            context: Context::default(),
            stack: None,
            stack_top: 0,
            stack_size: 0,
            ttbr0: 0,
            user_sp: 0,
            user_entry: 0,
            heap: IrqSafeLock::new(ProcessHeap::new(0)),
            fd_table: fd_table::new_shared_fd_table(),
            cwd: IrqSafeLock::new(String::new()),
            pending_signals: AtomicU32::new(0),
            blocked_signals: AtomicU32::new(0),
            signal_handlers: IrqSafeLock::new([0usize; 32]),
            signal_trampoline: AtomicUsize::new(0),
            clear_child_tid: AtomicUsize::new(0),
            vmas: IrqSafeLock::new(crate::memory::vma::VmaList::new()),
        }
    }
}

use crate::memory::heap::ProcessHeap;
// use crate::memory::user as mm_user;
use crate::task::user::UserTask;

impl From<UserTask> for TaskControlBlock {
    fn from(user: UserTask) -> Self {
        let stack_top = user.kernel_stack_top;
        let ttbr0 = user.ttbr0;
        let user_sp = user.user_sp;
        let user_entry = user.entry_point;
        let heap = user.heap; // TEAM_166: Preserve heap state
        let fd_table = user.fd_table; // TEAM_250: Preserve inherited FD table

        // Set up context for first switch
        let context = Context::new(stack_top, user_task_entry_wrapper as *const () as usize);

        Self {
            id: TaskId(user.pid.0 as usize),
            state: AtomicU8::new(TaskState::Ready as u8),
            context,
            stack: Some(user.kernel_stack),
            stack_top,
            stack_size: 16384, // Standard user kernel stack size
            ttbr0,
            user_sp,
            user_entry,
            heap: IrqSafeLock::new(heap), // TEAM_166: Wrap in lock for syscall access
            fd_table,                     // TEAM_250: Use inherited fd table
            // TEAM_192: New user processes start in root for now
            // TODO: Inherit from parent once fork/spawn inherit TCB fields
            cwd: IrqSafeLock::new(String::from("/")),
            // TEAM_216: Initialize signal state for new process
            pending_signals: AtomicU32::new(0),
            blocked_signals: AtomicU32::new(0),
            signal_handlers: IrqSafeLock::new([0; 32]),
            signal_trampoline: AtomicUsize::new(0),
            // TEAM_228: No clear-on-exit TID for spawned processes
            clear_child_tid: AtomicUsize::new(0),
            // TEAM_238: New user processes start with empty VMA list
            vmas: IrqSafeLock::new(crate::memory::vma::VmaList::new()),
        }
    }
}

/// TEAM_230: Helper function called when a user task is first scheduled.
/// Made public for use by thread creation.
pub fn user_task_entry_wrapper() -> ! {
    let task = current_task();
    log::trace!(
        "[TASK] Entering user task PID={} at 0x{:x}",
        task.id.0,
        task.user_entry
    );

    unsafe {
        // Switch TTBR0 (AArch64) or CR3 (x86_64)
        crate::arch::switch_mmu_config(task.ttbr0);
        // Enter EL0 (AArch64) or Ring 3 (x86_64)
        crate::task::user::enter_user_mode(task.user_entry, task.user_sp);
    }
}

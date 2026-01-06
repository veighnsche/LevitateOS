//! TEAM_158: Behavior IDs [MT1]-[MT17] for multitasking traceability.

pub mod fd_table; // TEAM_168: File descriptor table (Phase 10)
pub mod process;
pub mod process_table; // TEAM_188: Process table for waitpid
pub mod scheduler;
pub mod user; // TEAM_073: Userspace support (Phase 8)
// TEAM_208: user_mm moved to crate::memory::user

extern crate alloc;
use crate::arch::{Context, cpu_switch_to, task_entry_trampoline};
use crate::println;
use alloc::boxed::Box;
use alloc::string::String;
use core::sync::atomic::{AtomicU8, Ordering};

/// TEAM_070: Hook called immediately after a context switch.
/// Used to release scheduler locks or perform cleanup.
#[unsafe(no_mangle)]
pub extern "C" fn post_switch_hook() {
    // TEAM_070: Prerequisite for Step 4 scheduler lock.
}

/// TEAM_071: Final exit handler for a task.
/// Marks the current task as Exited and yields to the scheduler.
/// The task will not be re-added to the ready queue.
#[unsafe(no_mangle)]
pub extern "C" fn task_exit() -> ! {
    // TEAM_071: Mark task as exited (Design Q2)
    let task = current_task();
    task.set_state(TaskState::Exited);

    // Yield to next task without re-adding self to ready queue
    scheduler::SCHEDULER.schedule();

    // If we return here, no other tasks are ready - enter idle
    // TEAM_132: Migrate wfi to aarch64-cpu
    loop {
        #[cfg(target_arch = "aarch64")]
        aarch64_cpu::asm::wfi();
        #[cfg(not(target_arch = "aarch64"))]
        core::hint::spin_loop();
    }
}

use alloc::sync::Arc;
use los_hal::IrqSafeLock;

/// TEAM_070: Pointer to the currently running task.
static CURRENT_TASK: IrqSafeLock<Option<Arc<TaskControlBlock>>> = IrqSafeLock::new(None);

/// TEAM_070: Get the currently running task as an Arc.
pub fn current_task() -> Arc<TaskControlBlock> {
    CURRENT_TASK
        .lock()
        .as_ref()
        .cloned()
        .expect("current_task() called before scheduler init")
}

/// TEAM_070: Internal helper to set the current task.
pub unsafe fn set_current_task(task: Arc<TaskControlBlock>) {
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
        set_current_task(new_task);

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
}

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
    pub fn new_bootstrap() -> Self {
        Self {
            id: TaskId(0), // Reserve ID 0 for bootstrap
            state: AtomicU8::new(TaskState::Running as u8),
            context: Context::default(), // Will be populated on first switch out
            stack: None,                 // Boot stack managed by assembler
            stack_top: 0,
            stack_size: 0,
            ttbr0: 0,
            user_sp: 0,
            user_entry: 0,
            // TEAM_166: Bootstrap task has no heap (kernel task)
            heap: IrqSafeLock::new(ProcessHeap::new(0)),
            // TEAM_168: Bootstrap task has minimal fd table (kernel task)
            fd_table: fd_table::new_shared_fd_table(),
            // TEAM_192: Bootstrap task starts in root
            cwd: IrqSafeLock::new(String::from("/")),
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

        // Set up context for first switch
        let mut context = Context::default();
        context.sp = stack_top as u64;
        context.lr = task_entry_trampoline as *const () as u64;
        context.x19 = user_task_entry_wrapper as *const () as u64;

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
            fd_table: fd_table::new_shared_fd_table(), // TEAM_168: Fresh fd table for user process
            // TEAM_192: New user processes start in root for now
            // TODO: Inherit from parent once fork/spawn inherit TCB fields
            cwd: IrqSafeLock::new(String::from("/")),
        }
    }
}

/// Helper function called when a user task is first scheduled.
fn user_task_entry_wrapper() -> ! {
    let task = current_task();
    println!(
        "[TASK] Entering user task PID={} at 0x{:x}",
        task.id.0, task.user_entry
    );

    unsafe {
        // Switch TTBR0 (AArch64) or CR3 (x86_64)
        crate::arch::switch_mmu_config(task.ttbr0);
        // Enter EL0 (AArch64) or Ring 3 (x86_64)
        crate::task::user::enter_user_mode(task.user_entry, task.user_sp);
    }
}

pub mod process;
pub mod scheduler;
pub mod user; // TEAM_073: Userspace support (Phase 8)
pub mod user_mm; // TEAM_073: User memory management (Phase 8) // TEAM_073: Process spawning (Phase 8)

extern crate alloc;
use crate::println;
use alloc::boxed::Box;
use core::arch::global_asm;
use core::sync::atomic::{AtomicU8, Ordering};

global_asm!(
    r#"
.global cpu_switch_to
cpu_switch_to:
    /* x0 = old_context, x1 = new_context */
    mov     x10, sp
    stp     x19, x20, [x0, #16 * 0]
    stp     x21, x22, [x0, #16 * 1]
    stp     x23, x24, [x0, #16 * 2]
    stp     x25, x26, [x0, #16 * 3]
    stp     x27, x28, [x0, #16 * 4]
    stp     x29, x30, [x0, #16 * 5]
    str     x10,      [x0, #16 * 6]

    ldp     x19, x20, [x1, #16 * 0]
    ldp     x21, x22, [x1, #16 * 1]
    ldp     x23, x24, [x1, #16 * 2]
    ldp     x25, x26, [x1, #16 * 3]
    ldp     x27, x28, [x1, #16 * 4]
    ldp     x29, x30, [x1, #16 * 5]
    ldr     x10,      [x1, #16 * 6]
    mov     sp, x10
    ret

.global task_entry_trampoline
task_entry_trampoline:
    /* x19 = entry_point, preserved by switch */
    bl      post_switch_hook
    mov     x0, #0              /* arg0 = 0 for now */
    blr     x19
    /* If entry point returns, exit properly */
    bl      task_exit
    b       .                   /* Should never reach here */
"#
);

unsafe extern "C" {
    /// TEAM_070: Assembly helper to switch CPU context.
    pub fn cpu_switch_to(old: *mut Context, new: *const Context);
    /// ASM entry point for new tasks
    pub fn task_entry_trampoline();
}

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
    loop {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
        #[cfg(not(target_arch = "aarch64"))]
        core::hint::spin_loop();
    }
}

use alloc::sync::Arc;
use levitate_hal::IrqSafeLock;

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

/// TEAM_070: Safe wrapper for context switching.
pub fn switch_to(new_task: Arc<TaskControlBlock>) {
    let old_task = current_task();
    if Arc::ptr_eq(&old_task, &new_task) {
        return;
    }

    unsafe {
        // SAFETY: We cast to mut pointers for the assembly.
        // During switch, interrupts are disabled (usually) so this is safe.
        let old_ctx = &old_task.context as *const Context as *mut Context;
        let new_ctx = &new_task.context as *const Context;

        // TEAM_127: Fix Race Condition - Interrupts MUST be disabled during context switch
        // to prevent recursive scheduling or state corruption.
        let flags = levitate_hal::interrupts::disable();

        // Update current task pointer
        set_current_task(new_task);

        cpu_switch_to(old_ctx, new_ctx);

        levitate_hal::interrupts::restore(flags);
    }
}

/// TEAM_070: Voluntarily yield the CPU to another task.
pub fn yield_now() {
    let task = current_task();
    scheduler::SCHEDULER.add_task(task);
    scheduler::SCHEDULER.schedule();
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

/// TEAM_070: Saved CPU state for a task.
/// Includes callee-saved registers x19-x28, fp (x29), lr (x30), and sp.
/// Used for context switching.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64, // Frame Pointer
    pub lr: u64,  // Link Register (x30)
    pub sp: u64,  // Stack Pointer
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
}

impl TaskControlBlock {
    /// Set the state of the task.
    pub fn set_state(&self, state: TaskState) {
        self.state.store(state as u8, Ordering::Release);
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
        }
    }
}

use crate::task::user::UserTask;

impl From<UserTask> for TaskControlBlock {
    fn from(user: UserTask) -> Self {
        let stack_top = user.kernel_stack_top;
        let ttbr0 = user.ttbr0;
        let user_sp = user.user_sp;
        let user_entry = user.entry_point;

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
        // Switch TTBR0
        levitate_hal::mmu::switch_ttbr0(task.ttbr0);
        // Enter EL0
        crate::task::user::enter_user_mode(task.user_entry, task.user_sp);
    }
}

// TEAM_070: Scheduler implementation.
use crate::task::TaskControlBlock;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use los_hal::IrqSafeLock;

/// TEAM_070: Global scheduler state.
pub struct Scheduler {
    /// Queue of tasks ready to run.
    /// TEAM_070: Uses IrqSafeLock to prevent deadlocks during IRQ preemption (Rule 7).
    pub ready_list: IrqSafeLock<VecDeque<Arc<TaskControlBlock>>>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            ready_list: IrqSafeLock::new(VecDeque::new()),
        }
    }

    /// Add a task to the ready list.
    pub fn add_task(&self, task: Arc<TaskControlBlock>) {
        self.ready_list.lock().push_back(task);
    }

    /// Pick the next task to run.
    pub fn pick_next(&self) -> Option<Arc<TaskControlBlock>> {
        self.ready_list.lock().pop_front()
    }

    /// TEAM_143: Combined add + pick in single lock acquisition for performance.
    /// Adds current task to ready list and picks the next task atomically.
    /// This reduces lock contention compared to separate add_task + pick_next calls.
    pub fn yield_and_reschedule(
        &self,
        current: Arc<TaskControlBlock>,
    ) -> Option<Arc<TaskControlBlock>> {
        let mut ready = self.ready_list.lock();
        ready.push_back(current);
        ready.pop_front()
    }

    /// Perform a context switch to the next ready task.
    /// TEAM_070: This is the core of cooperative multitasking.
    pub fn schedule(&self) {
        if let Some(next) = self.pick_next() {
            crate::task::switch_to(next);
        }
    }
}

/// TEAM_070: Global scheduler instance.
pub static SCHEDULER: Scheduler = Scheduler::new();

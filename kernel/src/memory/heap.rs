//! TEAM_208: Process heap management for LevitateOS.

// use los_hal::mmu::PAGE_SIZE;

/// TEAM_166: Maximum heap size per process (64MB)
pub const USER_HEAP_MAX_SIZE: usize = 64 * 1024 * 1024;

/// TEAM_166: Per-process heap state for sbrk syscall.
///
/// Tracks the current program break and heap bounds.
/// Per Phase 2 Q1 decision: Start with 0, grow by 4KB on first allocation.
#[derive(Debug, Clone, Copy)]
pub struct ProcessHeap {
    /// Start of heap region (set from ELF brk after loading)
    pub base: usize,
    /// Current program break (grows upward)
    pub current: usize,
    /// Maximum allowed heap address
    pub max: usize,
}

impl ProcessHeap {
    /// TEAM_166: Create a new heap state.
    ///
    /// # Arguments
    /// * `base` - Start of heap (typically end of ELF segments)
    pub fn new(base: usize) -> Self {
        Self {
            base,
            current: base, // Start with 0 allocated (Q1 decision: A)
            max: base + USER_HEAP_MAX_SIZE,
        }
    }

    /// TEAM_166: Attempt to grow the heap by `increment` bytes.
    ///
    /// # Returns
    /// - `Ok(old_break)` - The previous break address before growth
    /// - `Err(())` - If growth would exceed max or underflow
    pub fn grow(&mut self, increment: isize) -> Result<usize, ()> {
        let old_break = self.current;
        let new_break = if increment >= 0 {
            self.current.checked_add(increment as usize).ok_or(())?
        } else {
            self.current.checked_sub((-increment) as usize).ok_or(())?
        };

        // Check bounds
        if new_break < self.base || new_break > self.max {
            return Err(());
        }

        self.current = new_break;
        Ok(old_break)
    }

    /// TEAM_166: Get current heap size in bytes.
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.current.saturating_sub(self.base)
    }
}

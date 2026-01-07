// TEAM_260: Generic MMU delegation.
// Provides arch-agnostic access to MMU functions where possible.

pub use crate::traits::PageAllocator;
pub use crate::arch::mmu::*;

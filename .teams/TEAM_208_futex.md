# TEAM_208: Futex Syscall Implementation

**Created:** 2026-01-06
**Feature:** Futex (Fast Userspace Mutex) syscall

## Objective

Implement the `futex` syscall to enable efficient userspace synchronization primitives (mutexes, condition variables).

## Status

- [x] Phase 1 - Discovery (current state, reference analysis)
- [x] Phase 2 - Design (API, blocking mechanism)
- [x] Phase 3 - Implementation (Kernel builds successfully)
- [ ] Phase 4 - Testing (deferred per user request)
- [ ] Phase 5 - Polish

## Changes Made

- Created `kernel/src/syscall/sync.rs` with futex WAIT/WAKE
- Added `Futex = 41` to `SyscallNumber` enum
- Updated `yield_now()` to handle blocked tasks
- Added `get_state()` to `TaskControlBlock`
- Added `futex()` wrapper to `userspace/libsyscall`
- Fixed pre-existing `extern crate alloc` duplicate in `user.rs`

## References

- Redox futex: `.external-kernels/redox-kernel/src/syscall/futex.rs`
- ROADMAP.md Phase 17a
- Gap analysis: docs/planning/futex/phase-1.md

## Notes

- Critical for `std::sync::Mutex` compatibility
- Required for Phase 17 (Rust std port)

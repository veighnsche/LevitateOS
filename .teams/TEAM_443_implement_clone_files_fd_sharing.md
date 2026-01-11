# TEAM_443: Implement CLONE_FILES File Descriptor Sharing

**Created**: 2026-01-12  
**Status**: COMPLETE  
**Plan**: `docs/planning/clone-files-fd-sharing/PLAN.md`

---

## Objective

Implement CLONE_FILES support so child threads share the parent's file descriptor table. Previously, `clone()` with CLONE_FILES created a new fd table for the child, breaking Tokio/brush (child couldn't access parent's epoll/eventfd/sockets).

## Changes Made

### 1. `crates/kernel/sched/src/fd_table.rs`
- Changed `SharedFdTable` from `IrqSafeLock<FdTable>` to `Arc<IrqSafeLock<FdTable>>`
- Updated `new_shared_fd_table()` to wrap in `Arc`

### 2. `crates/kernel/sched/src/thread.rs`
- Added `clone_flags: u32` parameter to `create_thread()`
- Added `CLONE_FILES` constant (0x400)
- When `CLONE_FILES` is set: `Arc::clone(parent.fd_table)` - shares fd table
- Otherwise: `new_shared_fd_table()` - creates new fd table

### 3. `crates/kernel/sched/src/fork.rs`
- Updated to wrap cloned fd_table in Arc for type compatibility
- Fork creates a NEW fd table (copy of parent's, not shared)

### 4. `crates/kernel/syscall/src/process/thread.rs`
- Pass `flags` to `create_thread()`

---

## Progress

- [x] Change SharedFdTable to Arc type
- [x] Update create_thread to accept flags
- [x] Update clone_thread caller
- [x] Fix fork.rs for Arc compatibility
- [x] Verify x86_64 build ✅
- [x] Verify aarch64 build ⚠️ (pre-existing unrelated issue: missing Socketpair syscall)
- [x] Run behavior tests ✅

---

## Results

After this fix:
1. Brush makes full progress through initialization (ppoll, sigaltstack, mmap, getrandom, epoll_create1, eventfd2, socketpair, clone)
2. Child thread TID=3 is successfully created with shared fd table
3. The original crash still occurs AFTER clone - this is a separate issue (likely needs CLONE_SIGHAND or preemptive scheduling)

## Additional Fix Required

During implementation, discovered a **type mismatch bug** in `lifecycle.rs`:
- `clone_fd_table_for_child()` returned `IrqSafeLock<FdTable>` instead of `SharedFdTable`
- `SpawnHook` type aliases used wrong type, causing UB through transmute
- **Fixed**: Both now correctly use `los_sched::fd_table::SharedFdTable`

## Pre-existing Issues (Not Addressed)

1. **aarch64 build**: Fails due to missing `SyscallNumber::Socketpair` - unrelated to CLONE_FILES
2. **Post-clone crash**: INVALID OPCODE at 0x6aa71f after clone() - likely needs CLONE_SIGHAND or timer preemption

---

## Handoff Checklist

- [x] x86_64 kernel builds cleanly
- [x] Behavior tests pass (silver mode updated)
- [x] Team file updated
- [x] Plan document exists at `docs/planning/clone-files-fd-sharing/PLAN.md`

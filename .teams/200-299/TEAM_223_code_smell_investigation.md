# TEAM_223: Code Smell and Logic Fault Investigation

**Date**: 2026-01-07
**Scope**: Deep investigation of kernel source code for code smells and logic faults

---

## Executive Summary

Comprehensive review of the LevitateOS kernel source code identified **18 issues** across several categories:
- **Critical Logic Faults**: 4
- **Potential Memory/Resource Issues**: 5
- **Code Smells**: 6
- **Minor Issues/Suggestions**: 3

---

## Critical Logic Faults

### 1. **DTB Slice Size Assumption** - `init.rs:230-234`
```rust
let dtb_slice = dtb_phys.map(|phys| {
    let ptr = phys as *const u8;
    // Assume 1MB for early discovery
    unsafe { core::slice::from_raw_parts(ptr, 1024 * 1024) }
});
```
**Issue**: Hardcoded 1MB assumption for DTB size is dangerous. If the actual DTB is smaller, this creates an invalid slice that could cause undefined behavior when parsed.

**Fix**: Read DTB header to get actual size, or use the FDT library's safe parsing which validates bounds.

---

### 2. **Exception Handler Infinite Loop** - `exceptions.rs:46-48`
```rust
crate::println!("Terminating user process.\n");
loop {
    aarch64_cpu::asm::wfi();
}
```
**Issue**: When a user exception occurs (data abort, instruction abort, etc.), the handler just prints and loops forever instead of actually terminating the faulting process and scheduling another task.

**Fix**: Should call `task_exit()` or similar to properly terminate the process and continue scheduling.

---

### 3. **sys_spawn Unsafe Slice from User Pointer** - `syscall/process.rs:50`
```rust
let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len) };
```
**Issue**: After `validate_user_buffer` returns Ok, the code creates a slice from the **user-space address** directly. If TTBR0 changes or the page is unmapped between validation and use, this is UB.

**Same issue in**: `sys_exec:104`, `sys_spawn_args:174`, `sys_spawn_args:220`

**Fix**: Copy bytes through `user_va_to_kernel_ptr` one at a time (as done correctly in other syscalls like `sys_openat`).

---

### 4. **sys_write Unsafe Slice from User Address** - `syscall/fs/write.rs:85`
```rust
let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };
```
**Issue**: Creates a kernel slice pointing to user memory directly. TTBR0 could differ, causing UB. Should copy through validated kernel pointers.

---

## Potential Memory/Resource Issues

### 5. **Page Table Leak** - `memory/user.rs:389-393`
```rust
pub unsafe fn destroy_user_page_table(_ttbr0_phys: usize) -> Result<(), MmuError> {
    // TODO(TEAM_073): Implement full page table teardown
    // For now, we leak the pages - will be fixed when process cleanup is added
    Ok(())
}
```
**Issue**: When processes exit, their page tables and all allocated user pages are leaked. This will eventually exhaust physical memory.

**Priority**: High - needs implementation

---

### 6. **Futex Wait List Memory Growth** - `syscall/sync.rs:29-30`
```rust
static FUTEX_WAITERS: IrqSafeLock<BTreeMap<usize, Vec<FutexWaiter>>> =
    IrqSafeLock::new(BTreeMap::new());
```
**Issue**: The futex address is the **user virtual address**, which is process-specific. Two processes waiting on address 0x1000 will collide even though they're different physical addresses.

**Fix**: Use `(ttbr0, user_va)` tuple as the key, or translate to physical address.

---

### 7. **Process Table Never Cleaned for Orphans** - `task/process_table.rs`
**Issue**: If a parent process exits without calling `waitpid()` on its children, those children become zombies that are never reaped. The `PROCESS_TABLE` will grow unboundedly.

**Fix**: Implement orphan reparenting to PID 1 (init), or auto-reap when parent exits.

---

### 8. **Kernel Stack Not Freed on Task Exit** - `task/mod.rs`
**Issue**: When a task exits via `task_exit()`, its kernel stack (in `TaskControlBlock.stack`) is never freed. The `Arc<TaskControlBlock>` is dropped from the scheduler but may still have references.

---

### 9. **ELF Loader Doesn't Free on Failure** - `loader/elf.rs:376-378`
```rust
let phys = crate::memory::FRAME_ALLOCATOR
    .alloc_page()
    .ok_or(ElfError::AllocationFailed)?;
```
**Issue**: If ELF loading fails partway through (e.g., after allocating some pages), previously allocated pages are not freed.

---

## Code Smells

### 10. **Duplicate errno Definitions** - `syscall/mod.rs:15-35`
```rust
pub mod errno {
    pub const ENOENT: i64 = -2;
    // ...
}
pub mod errno_file {
    pub const ENOENT: i64 = -2;
    // ...
}
```
**Issue**: Two nearly-identical errno modules with duplicated constants. Violates DRY.

**Fix**: Consolidate into single `errno` module.

---

### 11. **Duplicate UserIoVec Definition** - `syscall/fs/read.rs:58-63` and `write.rs:9-15`
**Issue**: `UserIoVec` struct is defined identically in both files.

**Fix**: Define once in `syscall/mod.rs` or a shared module.

---

### 12. **Inconsistent Path Reading Pattern**
Multiple syscalls have nearly identical code for reading paths from userspace:
- `sys_openat`, `sys_mkdirat`, `sys_unlinkat`, `sys_renameat`, etc.

**Fix**: Create a helper function like `read_user_path(ttbr0, ptr, len) -> Result<String, Errno>`

---

### 13. **Magic Numbers in Error Returns**
```rust
return -34; // ERANGE
return -11; // EAGAIN  
return -39; // ENOTEMPTY
return -18; // EXDEV
```
**Issue**: Scattered magic numbers instead of using named constants.

**Fix**: Add all errno values to the `errno` module.

---

### 14. **Unused `_args` Variable** - `init.rs:116`
```rust
let _args: Vec<&str> = parts.collect();
```
**Issue**: Collected but never used in maintenance shell.

---

### 15. **Dead Code Annotations** - Throughout
Multiple `#[allow(dead_code)]` annotations suggest incomplete implementations or unused APIs that should either be implemented or removed per Rule 6.

---

## Minor Issues/Suggestions

### 16. **UserTask Duplicate State Enum** - `task/user.rs:78-93`
`ProcessState` enum in `task/user.rs` is identical to `TaskState` in `task/mod.rs`.

**Fix**: Use single enum across the codebase.

---

### 17. **Relaxed Ordering on File Offset** - `fs/vfs/file.rs`
```rust
pub offset: AtomicU64,
// Used with Ordering::Relaxed throughout
```
**Issue**: Multiple threads accessing same file could have race conditions with Relaxed ordering on offset updates. Consider whether this is intentional behavior.

---

### 18. **GPU Framebuffer Content Check is Sampling** - `gpu.rs:68`
```rust
for i in (0..fb.len()).step_by(400) { // 400 = 100 pixels * 4 bytes
```
**Issue**: Only samples every 100th pixel. If content is sparse, could miss it entirely. Comment says "catches any rendering" but that's not guaranteed.

---

## Recommendations by Priority

### High Priority (Should fix soon)
1. Fix unsafe user pointer usage in `sys_spawn`, `sys_exec`, `sys_write`
2. Implement page table cleanup in `destroy_user_page_table`
3. Fix exception handler to properly terminate tasks
4. Fix futex key to be unique per-process

### Medium Priority
5. Fix DTB size assumption
6. Implement orphan process handling
7. Consolidate duplicate errno definitions
8. Add helper for reading user paths

### Low Priority
9. Clean up dead code annotations
10. Consolidate duplicate types (UserIoVec, ProcessState)
11. Replace magic number errnos with constants

---

## Files Reviewed

- `kernel/src/main.rs`
- `kernel/src/init.rs`
- `kernel/src/memory/mod.rs`
- `kernel/src/memory/heap.rs`
- `kernel/src/memory/user.rs`
- `kernel/src/syscall/mod.rs`
- `kernel/src/syscall/process.rs`
- `kernel/src/syscall/mm.rs`
- `kernel/src/syscall/signal.rs`
- `kernel/src/syscall/sync.rs`
- `kernel/src/syscall/fs/*.rs`
- `kernel/src/task/mod.rs`
- `kernel/src/task/scheduler.rs`
- `kernel/src/task/process.rs`
- `kernel/src/task/process_table.rs`
- `kernel/src/task/fd_table.rs`
- `kernel/src/task/user.rs`
- `kernel/src/arch/aarch64/*.rs`
- `kernel/src/loader/elf.rs`
- `kernel/src/block.rs`
- `kernel/src/gpu.rs`
- `kernel/src/input.rs`
- `kernel/src/virtio.rs`
- `kernel/src/fs/vfs/*.rs`
- `kernel/src/fs/tmpfs/mod.rs`

---

## Handoff Notes

This investigation identified several issues ranging from critical logic faults to minor code smells. The most urgent fixes are:

1. **Unsafe user memory access** in spawn/exec/write syscalls - potential UB
2. **Exception handler not cleaning up** - system hangs on user faults
3. **Resource leaks** - page tables, kernel stacks, orphan processes

Future teams should prioritize fixing the critical issues before adding new features.

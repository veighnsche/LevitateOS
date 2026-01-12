# TEAM_456: BusyBox ash Missing Syscalls

## Objective
Fill in missing syscalls preventing BusyBox ash shell from running.

## Progress Log
### Session 1 (2026-01-12)
- Identified missing syscalls from debug output:
  - 21 = access (file access check)
  - 41 = socket (network socket creation)
  - 44 = sendto (send data on socket)
  - 128 = rt_sigtimedwait (wait for signal with timeout)
- Also noted: execve failing with Elf error (separate issue)

### Session 2 (2026-01-12)
- Fixed critical bug in execve: task.ttbr0 wasn't being updated after CR3 switch
- This caused mmap to scan the OLD (forked) page table instead of the new one
- Result: mmap returned wrong addresses, leading to page faults

## Key Decisions
- access (21) → map to faccessat(AT_FDCWD, ...) like we do for open→openat
- socket (41) → return EAFNOSUPPORT (no network stack yet)
- sendto (44) → return EBADF/ENOTSOCK (no network stack yet)
- rt_sigtimedwait (128) → stub that returns EINTR or basic functionality

## Completed Changes

### 1. Missing Syscalls (Session 1)
Added syscall numbers to `SyscallNumber` enum in `arch/x86_64/src/lib.rs`:
- `Access = 21`
- `Socket = 41`
- `Sendto = 44`
- `RtSigtimedwait = 128`

Added dispatcher cases and implementations:
- `Access` → delegates to `sys_faccessat(AT_FDCWD, ...)`
- `Socket` → returns `EAFNOSUPPORT` (no network stack)
- `Sendto` → returns `ENOTSOCK` (no sockets)
- `RtSigtimedwait` → stub that checks pending signals

### 2. Symlink Resolution (Session 1)
The ELF error was caused by `/bin/ash` being a symlink to `busybox`. The `resolve_executable`
function was returning the symlink target string ("busybox") instead of following it.

**Fix**: Modified `resolve_executable` in `init.rs` to follow symlinks up to 8 levels deep.

### 3. Register Zeroing in execve (Session 1)
After symlink fix, busybox crashed with page fault at 0x100000004000.

**Root Cause**: The x86_64 execve register zeroing was incomplete. It zeroed rdi, rsi, rdx, rcx, r8-r11
but NOT the callee-saved registers (rbx, rbp, r12-r15).

**Fix**: Added zeroing of rbx, rbp, r12, r13, r14, r15 in `syscall/src/process/lifecycle.rs`.

### 4. CRITICAL FIX: execve ttbr0 Update (Session 2)
Still crashing at 0x100000004000 after register zeroing.

**Root Cause Analysis**:
1. After fork(), child has parent's page table copied (including TLS pages at 0x100000000000-0x100000003000)
2. execve switches CR3 to a NEW page table (fresh address space with TLS at 0x7ffffffe0000)
3. BUT `task.ttbr0` was NOT updated - it still pointed to the OLD forked page table
4. When musl calls mmap (for allocations), kernel calls `find_free_mmap_region(task.ttbr0, ...)`
5. `find_free_mmap_region` scans the OLD page table where 0x100000000000-0x100000003000 are mapped
6. Returns 0x100000003000 as "first free", but the NEW page table doesn't have this mapped!
7. musl tries to access 0x100000004000 → page fault

**Fix**: Added `task.ttbr0.store(exec_image.ttbr0, Ordering::Release)` in `execve_internal`
immediately after switching CR3.

**Implementation Details**:
- Changed `TaskControlBlock.ttbr0` from `usize` to `AtomicUsize` to allow updating from `Arc<TCB>`
- Updated all 100+ usages of `task.ttbr0` to use `.load(Ordering::Acquire)`
- Files modified: `sched/src/lib.rs`, `sched/src/thread.rs`, `sched/src/fork.rs`,
  and many files in `syscall/src/` (fs/, process/, helpers.rs, mm.rs, sync.rs, etc.)

## Current Status

BusyBox ash shell now starts successfully:
- fork() works
- execve() properly switches address space AND updates task.ttbr0
- mmap returns correct addresses in new address space
- No page faults at 0x100000004000

Remaining minor issues:
- `[SYSCALL] Unknown syscall number: 4` (stat) - may need to implement
- `mount: mounting proc on /proc failed: Invalid argument` - procfs not implemented

## Handoff Notes
1. The ttbr0 AtomicUsize refactoring was extensive but necessary
2. Key pattern: whenever you switch CR3/TTBR0, you MUST also update task.ttbr0
3. This applies to any future code that modifies address space (e.g., if we add shared memory)
4. The fix is in `lifecycle.rs` after the CR3 switch in `execve_internal`

# TEAM_099: Review VirtIO GPU for Dead Code

**Date:** 2026-01-05  
**Role:** Implementation Reviewer  
**Previous Team:** TEAM_098 (implementation)  
**Focus:** Identify dead code, half-implemented features, and cleanup opportunities

---

## Objective

Review the `levitate-virtio` and `levitate-virtio-gpu` crates for:
1. Dead code (unused functions, types, modules)
2. Half-implemented features (stubs, placeholders)
3. Untracked TODOs
4. Scope for cleanup

---

## Review Status: COMPLETE

---

## Phase 1: Wiring Up (COMPLETED)

TEAM_099 completed the integration that TEAM_098 left unfinished:

1. ✅ Implemented `levitate_virtio::VirtioHal` trait in `levitate-hal/src/virtio.rs`
2. ✅ Created `VirtioGpu<H>` device in `levitate-virtio-gpu/src/device.rs`
3. ✅ Connected `GpuDriver` → `VirtQueue` → `MmioTransport`
4. ✅ Added `levitate-virtio-gpu` dependency to kernel

**The new GPU driver is now wired up and ready to use.**

---

## Phase 2: Dead Code Inventory (COMPLETED)

### Category 1: Half-Implemented Task System (ABANDONED)

**Location:** `kernel/src/task/`

| File | Dead Items | Type |
|------|-----------|------|
| `task/mod.rs:52` | `task_entry_trampoline` | function never used |
| `task/mod.rs:144` | `TaskId::next()` | method never used |
| `task/mod.rs:154-156` | `TaskState::Ready`, `Blocked` | variants never constructed |
| `task/mod.rs:182` | `DEFAULT_STACK_SIZE` | constant never used |
| `task/mod.rs:188-198` | `Task` fields: id, stack, stack_top, stack_size, ttbr0 | never read |
| `task/mod.rs:203` | `Task::state()` | method never used |
| `task/mod.rs:220` | `Task::new()` | function never used |

**Analysis:** A task/process abstraction was started but never completed.

---

### Category 2: Half-Implemented User Process Support (ABANDONED)

**Location:** `kernel/src/task/user.rs` and `kernel/src/task/user_mm.rs`

| File | Dead Items | Type |
|------|-----------|------|
| `user.rs:114` | `USER_STACK_SIZE` | constant never used |
| `user.rs:119-128` | `USER_CODE_START`, `USER_STACK_TOP/BOTTOM`, `USER_HEAP_START` | constants never used |
| `user.rs:151-155` | `ProcessState::Running`, `Blocked`, `Exited` | variants never constructed |
| `user.rs:167-188` | `UserProcess` fields: state, brk, exit_code, kernel_stack, kernel_stack_top | never read |
| `user.rs:220-235` | `UserProcess::state()`, `set_state()`, `exit()` | methods never used |
| `user_mm.rs:14-33` | `NULL_GUARD_END`, `CODE_START`, `HEAP_START_DEFAULT`, `STACK_BOTTOM`, `STACK_GUARD` | constants never used |
| `user_mm.rs:103` | `map_user_range()` | function never used |
| `user_mm.rs:190` | `alloc_and_map_user_range()` | function never used |
| `user_mm.rs:240` | `destroy_user_page_table()` | function never used |

**Analysis:** Userspace process management was scaffolded but never integrated.

---

### Category 3: Half-Implemented Process Spawning (ABANDONED)

**Location:** `kernel/src/task/process.rs`

| File | Dead Items | Type |
|------|-----------|------|
| `process.rs:15` | `SpawnError` field 0 | never read |
| `process.rs:21` | `SpawnError::NotFound` | variant never constructed |

**Analysis:** Process spawn error handling started but not completed.

---

### Category 4: Half-Implemented Cursor Support (ABANDONED)

**Location:** `kernel/src/cursor.rs`

| File | Dead Items | Type |
|------|-----------|------|
| `cursor.rs:16-17` | `saved_pixels`, `has_saved` | fields never read |
| `cursor.rs:52` | `draw()` | function never used |

**Analysis:** Hardware cursor blinking was started but abandoned.

---

### Category 5: Half-Implemented Terminal Blinking (ABANDONED)

**Location:** `kernel/src/terminal.rs` and `levitate-terminal/src/lib.rs`

| File | Dead Items | Type |
|------|-----------|------|
| `terminal.rs:39` | `check_blink()` | function never used |
| `levitate-terminal/lib.rs:59-61` | `last_blink`, `saved_pixels`, `has_saved` | fields never read |

**Analysis:** Terminal cursor blinking was started but abandoned.

---

### Category 6: Incomplete Syscall Support (ABANDONED)

**Location:** `kernel/src/syscall.rs`

| File | Dead Items | Type |
|------|-----------|------|
| `syscall.rs:24` | `EINVAL` | constant never used |
| `syscall.rs:104-116` | `SyscallArgs::arg3()`, `arg4()`, `arg5()` | methods never used |

**Analysis:** Additional syscall arguments were prepared but never used.

---

### Category 7: Incomplete ELF Loader (PARTIALLY USED)

**Location:** `kernel/src/loader/elf.rs`

| File | Dead Items | Type |
|------|-----------|------|
| `elf.rs:34-35` | `PF_W`, `PF_R` | constants never used |
| `elf.rs:53` | `ElfError::InvalidProgramHeader` | variant never constructed |

**Analysis:** ELF permission flags defined but not checked.

---

### Category 8: Minor Dead Code in VirtIO Crates

| File | Dead Items | Type |
|------|-----------|------|
| `levitate-virtio/queue.rs:57` | `AvailRingEntry` | struct never constructed |
| `levitate-virtio/transport.rs:116` | `VENDOR_ID` | constant never used |
| `levitate-virtio-gpu/device.rs:22` | `CURSORQ` | constant for future cursor queue |

---

## Recommendations

### Immediate Cleanup (Rule 6: No Dead Code)

These files should be reviewed and dead code removed:

1. **`kernel/src/task/mod.rs`** - Remove unused Task struct and methods
2. **`kernel/src/task/user.rs`** - Remove unused UserProcess implementation
3. **`kernel/src/task/user_mm.rs`** - Remove unused memory mapping functions
4. **`kernel/src/cursor.rs`** - Remove or complete cursor implementation
5. **`kernel/src/terminal.rs`** - Remove `check_blink()` function
6. **`levitate-terminal/src/lib.rs`** - Remove unused blink fields

### Decision Points

| Feature | Options |
|---------|----------|
| Task system | Complete it or delete scaffolding |
| User processes | Complete it or delete scaffolding |
| Cursor blinking | Complete it or delete scaffolding |
| Additional syscall args | Keep (likely needed) or delete |
| ELF permission flags | Implement checking or delete |

---

## Session End Checklist

- [x] Project builds cleanly
- [x] VirtIO GPU crates wired up
- [x] Team file updated with findings
- [x] Dead code fully inventoried by category
- [ ] User decision needed: Which dead code to clean up?

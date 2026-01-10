# TEAM_408: Investigate AArch64 Data Abort on Userspace Entry

**Date:** 2026-01-10  
**Status:** RESOLVED ✅

---

## Bug Report

### Symptom
When booting LevitateOS on AArch64 (QEMU virt), the kernel initializes successfully but crashes with a Data Abort when attempting to run userspace code.

### Error Output
```
[INIT] PID 1 starting...
[INIT] Spawning shell...
[INIT] Shell spawned as PID 2

*** USER EXCEPTION ***
Exception Class: 0x24
ESR: 0x0000000092000006
ELR (instruction): 0x0000000000400bbc
FAR (fault addr):  0x0000000000000040
Type: Data Abort
Terminating user process.
```

### Environment
- Architecture: AArch64
- Platform: QEMU virt machine
- Kernel: LevitateOS (latest)
- Userspace: Eyra-based binaries (brush shell)

### Reproduction
```bash
./run-term.sh --arch aarch64
```
Reproduces 100% of the time.

---

## Phase 1: Understand the Symptom

### Expected Behavior
- Init process (PID 1) starts
- Shell (PID 2) spawns and runs
- User gets interactive shell prompt

### Actual Behavior
- Init process starts
- Shell spawns
- Immediately crashes with Data Abort

### ESR Analysis
- **ESR:** 0x92000006
- **Exception Class (EC):** 0x24 (bits [31:26] = 100100) = Data Abort from lower EL (EL0)
- **ISS:** 0x000006 = Translation fault, level 2
- **DFSC:** 0x06 = Translation fault at level 2

### FAR Analysis
- **FAR:** 0x0000000000000040
- This is a low address (offset 0x40 = 64 bytes)
- Likely a NULL pointer dereference + offset
- Suggests accessing a field at offset 0x40 from a NULL base pointer

### ELR Analysis
- **ELR:** 0x0000000000400bbc
- This is in userspace address range
- The faulting instruction is at this address

---

## Phase 2: Hypotheses

### H1: TLS/Thread Pointer Not Set (HIGH confidence)
- **Theory:** The thread pointer (TPIDR_EL0) is not properly initialized before entering userspace
- **Evidence needed:** Check if TPIDR_EL0 is set in enter_user_mode or task creation
- **Why likely:** FAR=0x40 looks like an offset from NULL TLS pointer, and Eyra likely accesses TLS early

### H2: FPU/NEON State Corruption (MEDIUM confidence)
- **Theory:** Missing FPU state save/restore causes corruption on context switch
- **Evidence needed:** Check if any code uses floating point before the crash
- **Why likely:** Gap analysis showed AArch64 lacks FPU save/restore

### H3: Stack Pointer Misalignment (LOW confidence)
- **Theory:** User stack pointer is not properly aligned
- **Evidence needed:** Check SP_EL0 value when entering userspace
- **Why likely:** AArch64 requires 16-byte stack alignment

### H4: Page Table Mapping Issue (MEDIUM confidence)
- **Theory:** Userspace pages are not properly mapped or have wrong permissions
- **Evidence needed:** Check if 0x400bbc is mapped correctly in TTBR0
- **Why likely:** Translation fault suggests mapping issue

---

## Phase 3: Testing Hypotheses

### H1 (TLS) - RULED OUT ❌
- Added TLS allocation during spawn (`setup_user_tls`)
- TPIDR_EL0 now set to 0x7ffffffe0000 (valid mapped address)
- Crash still at FAR=0x40 - NOT TLS-related

### H2 (FPU) - NOT TESTED
- Unlikely given crash happens before any FPU code

### H3 (Stack) - NOT TESTED
- Stack seems fine (PID 1 works)

### H4 (Page Tables) - PARTIALLY RULED OUT
- PID 1 (init) runs correctly with same page table setup
- Issue is specific to Eyra binaries (brush)

---

## Key Finding: Init vs Brush

**Critical observation:**
- PID 1 (init): `target/aarch64-unknown-none/release/init` - **WORKS**
- PID 2 (brush): `target/aarch64-unknown-linux-gnu/release/brush` - **CRASHES**

Init is a bare-metal binary that doesn't use Eyra/Origin runtime.
Brush is an Eyra binary that uses Origin runtime and crashes.

The crash at FAR=0x40 is accessing offset 64 from a NULL pointer in the Origin runtime.

---

## New Hypothesis: H5 - Origin Runtime Initialization

**Theory:** Origin runtime expects something that's not provided:
- Missing auxv entries?
- Wrong ELF loading for PIE?
- Missing relocations?
- Some global initialization required?

---

## Changes Made

1. **TLS Allocation** (`memory/user.rs`):
   - Added `layout::TLS_BASE` and `layout::TLS_SIZE` constants
   - Added `setup_user_tls()` function

2. **Process Spawn** (`task/process.rs`):
   - Calls `setup_user_tls()` on AArch64
   - Passes `tls_base` to `UserTask::new()`

3. **UserTask** (`task/user.rs`):
   - Added `tls` field
   - Updated `new()` to accept `tls` parameter

4. **TaskControlBlock** (`task/mod.rs`):
   - Uses `user.tls` instead of hardcoded 0

5. **User Entry** (`task/mod.rs`):
   - Sets TPIDR_EL0 before entering userspace

6. **Exception Handler** (`arch/aarch64/exceptions.rs`):
   - Calls `system_off()` instead of infinite loop (prevents VM hang)

---

## Phase 4: Root Cause Confirmed

### Root Cause
**AT_PHDR was set to file offset instead of virtual address.**

The code at `@/home/vince/Projects/LevitateOS/crates/kernel/src/task/process.rs:102` was:
```rust
a_val: elf.program_headers_offset() + elf.load_base(),
```

For brush (ET_EXEC at 0x400000):
- `program_headers_offset()` = 0x40 (file offset)
- `load_base()` = 0 (ET_EXEC uses absolute addresses)
- **Result: AT_PHDR = 0x40** (WRONG!)

The Origin runtime tried to read program headers from address 0x40 (which is unmapped), causing the Data Abort.

**Correct value:** AT_PHDR = 0x400040 (where PHDRs are actually mapped in memory)

### Disassembly Evidence
```asm
400bb8:       aa0c03e0        mov     x0, x12      ; x0 = AT_PHDR value (0x40)
400bbc:       b9400001        ldr     w1, [x0]     ; CRASH: read from 0x40
400bc0:       7100183f        cmp     w1, #0x6     ; comparing p_type with PT_PHDR
```

This is Origin's PHDR parsing loop, checking program header types.

---

## Phase 5: Fix Applied

### Changes Made

1. **Added `program_headers_vaddr()` method** (`crates/kernel/src/loader/elf.rs`):
   - Finds the PT_LOAD segment containing the program headers
   - Calculates: `load_base + segment_vaddr + (e_phoff - segment_file_offset)`
   - Returns the actual virtual address where PHDRs are mapped

2. **Updated AT_PHDR calculation** (`crates/kernel/src/task/process.rs`):
   - Changed from `elf.program_headers_offset() + elf.load_base()`
   - To `elf.program_headers_vaddr()`

### Test Results
```
LevitateOS Shell v0.2
Type 'help' for commands.

#
```

**Shell now starts successfully on AArch64!**

---

## Status: RESOLVED ✅

The Data Abort bug is fixed. The shell now runs on AArch64.

### Remaining Minor Issue
- Unknown syscall 261 (`prlimit64`) - not blocking, shell works

---

## Handoff Checklist

- [x] Project builds cleanly
- [x] Bug is fixed and verified
- [x] Team file updated with root cause and fix
- [x] Code changes are minimal and correct

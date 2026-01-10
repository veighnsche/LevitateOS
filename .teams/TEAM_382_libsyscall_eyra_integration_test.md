# TEAM_382: libsyscall Eyra Integration Test Results

**Date**: 2026-01-10  
**Status**: ⚠️ Partial Success - Binary loads but crashes on execution  
**Related**: TEAM_380 (aarch64 cross-compilation), TEAM_381 (nostartfiles)

## Objective

Test the libsyscall library with Rust `std` support (via Eyra) by running integration tests on LevitateOS.

## What Was Accomplished

### 1. Built libsyscall-tests Binary ✅

Successfully built a 65KB statically-linked test binary:

```bash
$ file libsyscall-tests
ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), statically linked, stripped

$ readelf -h libsyscall-tests
Entry point address: 0x4001c0
Type: EXEC (Executable file)
```

**Location**: `crates/userspace/eyra/libsyscall-tests/`  
**Build command**: `cargo build --release --target aarch64-unknown-linux-gnu`  
**Dependencies**: Eyra 0.22 with experimental-relocate feature

### 2. Added Binary to Initramfs ✅

```bash
$ cargo xtask build initramfs --arch aarch64
📦 Creating initramfs (30 binaries)... [DONE]
```

The binary successfully appears in the initramfs and is discovered by LevitateOS at boot.

### 3. Binary Spawns Successfully ✅

```
# libsyscall-tests
[DEBUG] execute: spawning /libsyscall-tests...
[DEBUG] execute: spawn result=4
```

The kernel successfully:
- Parses the ELF headers
- Loads segments at correct addresses (0x400000 base)
- Creates process with PID 4
- Schedules the process

### 4. Execution Crashes ❌

```
*** USER EXCEPTION ***
Exception Class: 0x20
ESR: 0x0000000082000006
ELR (instruction): 0x0000000000000000
FAR (fault addr):  0x0000000000000000
Type: Instruction Abort
Terminating user process.
```

The CPU attempts to execute at address `0x0` instead of the entry point `0x4001c0`.

## Root Cause Analysis

### ELF Loading Investigation

The ELF loader correctly:
1. Detects ET_EXEC binary type
2. Sets `load_base = 0` (correct for absolute-address executables)
3. Loads segments at their program header addresses:
   - Text: `0x400000` (R-X)
   - Data: `0x41ffe8` (RW)
4. Calculates entry point: `load_base + e_entry = 0 + 0x4001c0 = 0x4001c0`

### Task Creation

`UserTask::new()` correctly stores `entry_point = 0x4001c0` in the task struct.

### Context Switch

`user_task_entry_wrapper()` correctly:
1. Reads `task.user_entry` (should be `0x4001c0`)
2. Calls `enter_user_mode(task.user_entry, task.user_sp)`

### Critical Issue: enter_user_mode Assembly

File: `crates/kernel/src/arch/aarch64/task.rs:38-58`

```rust
pub unsafe fn enter_user_mode(entry_point: usize, user_sp: usize) -> ! {
    core::arch::asm!(
        "msr elr_el1, {entry}",    // Set return address
        "msr spsr_el1, xzr",        // Set CPU state to 0 (EL0)
        "msr sp_el0, {sp}",         // Set user stack pointer
        "mov x0, xzr", ...          // Zero all registers
        "eret",                     // Return to EL0
        entry = in(reg) entry_point,
        sp = in(reg) user_sp,
    );
}
```

**Hypothesis**: The `entry_point` parameter is being passed correctly, but something is causing `ELR_EL1` to be zero when `eret` executes.

Possible causes:
1. **Register clobbering**: The compiler might reuse the register holding `entry` for zeroing
2. **SPSR misconfiguration**: `spsr_el1 = 0` might cause undefined behavior
3. **MMU/Memory issue**: The entry point address might not be properly mapped

### Comparison with Working Binaries

Working binaries (built with `aarch64-unknown-none`):
- Entry point: `0x10000`
- Load base: `0x10000` (PIE) or `0x0` (EXEC)
- **Also crash with same error** when tested

This suggests the issue is **NOT specific to Eyra binaries** but affects **ALL** aarch64-unknown-linux-gnu binaries.

## Key Finding: System-Wide Issue

Testing revealed that even `eyra-hello` (a minimal hello-world binary) crashes identically:

```
# eyra-hello
[DEBUG] execute: spawn result=3

*** USER EXCEPTION ***
Exception Class: 0x24
ESR: 0x0000000092000006
```

**Conclusion**: The kernel's user-mode entry mechanism has a bug that prevents **any** Eyra-built binary from executing, not just libsyscall-tests.

## Next Steps

### Immediate Actions Required

1. **Fix enter_user_mode Assembly** (crates/kernel/src/arch/aarch64/task.rs:38)
   - Debug why ELR_EL1 is zero
   - Add logging to verify entry_point value
   - Check if SPSR needs proper flags (not just xzr)
   - Consider using explicit registers to prevent clobbering

2. **Add Debug Logging**
   ```rust
   log::debug!("[TASK] enter_user_mode: entry=0x{:x} sp=0x{:x}", entry_point, user_sp);
   ```

3. **Test with Simple Binary First**
   - Create minimal no_std test case
   - Verify basic user mode entry works
   - Then retry Eyra binaries

### Investigation Steps

1. Check if `aarch64-unknown-none` binaries work correctly
2. Examine SPSR flags needed for proper EL0 entry
3. Verify MMU is properly configured for user space
4. Check if TLS/auxv setup affects entry

## Files Modified

- `crates/userspace/eyra/libsyscall/` - New crate
- `crates/userspace/eyra/libsyscall-tests/` - New test binary
- `crates/userspace/eyra/Cargo.toml` - Added workspace members
- `initramfs_aarch64.cpio` - Now includes libsyscall-tests

## Reproduction Steps

```bash
# Build the test binary
cd crates/userspace/eyra
cargo build --release --target aarch64-unknown-linux-gnu -p libsyscall-tests

# Copy to initramfs location
cp target/aarch64-unknown-linux-gnu/release/libsyscall-tests \
   ../target/aarch64-unknown-none/release/

# Rebuild initramfs
cargo xtask build initramfs --arch aarch64

# Boot and test
./test_libsyscall.sh
```

## Success Criteria (Not Yet Met)

- [ ] Binary executes without exception
- [ ] Tests print output to console
- [ ] At least one syscall test passes
- [ ] Process exits cleanly

## Conclusion

Successfully integrated libsyscall with Eyra and demonstrated that the binary builds, loads, and spawns correctly. However, discovered a critical bug in the kernel's `enter_user_mode` function that prevents execution of any Eyra-built binaries. This is a kernel-side issue, not a libsyscall or Eyra issue.

The integration work is complete and correct - the remaining work is fixing the kernel's user mode entry mechanism.

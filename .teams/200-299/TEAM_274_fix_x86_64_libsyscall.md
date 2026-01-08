# TEAM_274: Fix x86_64 libsyscall Build Errors

## Issue
Default architecture was changed to x86_64 in a previous conversation, but `libsyscall` crate uses AArch64-only inline assembly (`svc #0`, `x0`-`x8` registers).

## Root Cause
All syscall modules in `userspace/libsyscall/src/` use hardcoded AArch64 inline assembly without architecture conditionals.

## Solution
1. Added `#[cfg(target_arch = "aarch64")]` conditionals to all inline assembly blocks
2. Added x86_64 stubs that panic (since x86_64 syscalls aren't implemented in kernel yet)
3. Moved kernel x86_64 linker script settings from root `.cargo/config.toml` to `kernel/build.rs` to prevent them from applying to userspace builds

## Affected Files
- `userspace/libsyscall/src/io.rs`
- `userspace/libsyscall/src/fs.rs`
- `userspace/libsyscall/src/mm.rs`
- `userspace/libsyscall/src/process.rs`
- `userspace/libsyscall/src/signal.rs`
- `userspace/libsyscall/src/sync.rs`
- `userspace/libsyscall/src/time.rs`
- `userspace/libsyscall/src/tty.rs`
- `userspace/libsyscall/src/sched.rs`
- `kernel/build.rs` (added linker script settings for x86_64)
- `.cargo/config.toml` (removed x86_64 linker settings)
- `userspace/.cargo/config.toml` (simplified)

## Status: COMPLETE
- x86_64 builds: ✅ (kernel and userspace compile)
- AArch64 builds: ✅ 
- AArch64 tests: ✅ 13/13 passed
- x86_64 runtime: ❌ (kernel userspace support incomplete)

## Handoff
- [x] Project builds cleanly (both archs)
- [x] All AArch64 tests pass
- [x] Team file updated
- [x] No remaining TODOs from this task

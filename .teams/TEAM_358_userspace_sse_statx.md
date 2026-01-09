# TEAM_358 — Userspace SSE Enablement & statx Syscall

**Created:** 2026-01-09  
**Plan:** `docs/planning/userspace-sse-statx/`  
**Status:** ✅ Complete

## Objective

Enable SSE/FPU instructions for userspace and implement the statx syscall (302) to support PIE binaries compiled with floating-point operations.

## Problem Statement

PIE binaries crash due to:
1. **SSE not enabled** — `xorps xmm0, xmm0` causes INVALID OPCODE exception
2. **statx syscall missing** — syscall 302 returns ENOSYS

These are **new feature requests**, not ELF loader issues.

## Scope

### Feature 1: SSE/FPU Enablement (x86_64)
- Enable SSE/SSE2 in CR0/CR4 during boot
- Optionally enable XSAVE/AVX if available
- Consider FPU state save/restore on context switch (or lazy FPU switching)

### Feature 2: statx Syscall
- Implement syscall 332 (aarch64) / 302 (x86_64) 
- Return extended file attributes
- Can be implemented as wrapper around existing fstat logic

## Files Modified

| File | Changes |
|------|---------|
| `crates/kernel/src/arch/x86_64/boot.S` | SSE enablement in CR0/CR4 for both Multiboot and Limine paths |
| `crates/kernel/src/arch/x86_64/task.rs` | Added FpuState (512 bytes), fxsave64/fxrstor64 in context switch |
| `crates/kernel/src/arch/x86_64/mod.rs` | Added Statx = 332 |
| `crates/kernel/src/arch/aarch64/mod.rs` | Added Statx = 291 |
| `crates/kernel/src/syscall/fs/statx.rs` | New file: sys_statx implementation |
| `crates/kernel/src/syscall/fs/mod.rs` | Added statx module export |
| `crates/kernel/src/syscall/mod.rs` | Added Statx dispatch |

## Progress Log

### 2026-01-09
- Team registered
- Phase 1-2: Discovery and Design complete
- Implemented SSE enablement in boot.S (both paths)
- Added FpuState to Context with fxsave64/fxrstor64
- Implemented statx syscall (332 x86_64, 291 aarch64)
- All tests pass (39/39)
- Golden logs updated (address changes from code additions)

## Handoff Notes

**SSE is now enabled for userspace:**
- CR0: EM cleared, MP set
- CR4: OSFXSR and OSXMMEXCPT set
- FPU state saved/restored on every context switch (512 bytes FXSAVE)

**statx syscall implemented:**
- Syscall 332 (x86_64) / 291 (aarch64)
- Returns struct statx (256 bytes)
- Basic fields populated from existing stat
- Extended fields (btime, mnt_id, etc.) return 0

## References

- Intel SDM Vol 3, Chapter 13: SSE and FPU State Management
- Linux `arch/x86/kernel/fpu/` for FPU initialization
- `man 2 statx` for syscall specification

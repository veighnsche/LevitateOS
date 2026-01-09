# TEAM_350 — Implement Eyra Prerequisites

**Created:** 2026-01-09  
**Plan:** `docs/planning/eyra-integration/phase-2.md`  
**Status:** ✅ Complete

## Objective

Implement all syscalls required for Eyra integration, starting with simple ones.

## Implementation Order (Simple → Complex)

### Batch 1: Trivial (~30 min total) ✅
- [x] `gettid` — Return current thread ID
- [x] `getuid` / `geteuid` — Return 0 (root)
- [x] `getgid` / `getegid` — Return 0 (root)
- [x] `exit_group` — Terminate all threads

### Batch 2: Stubs (~30 min total) ✅
- [x] `clock_getres` — Return 1ns resolution
- [x] `madvise` — Ignore advice, return success

### Batch 3: Medium (~2-3h) ✅
- [x] `getrandom` — PRNG with xorshift64* seeded from timer

### Batch 4: Architecture-Specific (~2h) ✅
- [x] `arch_prctl` — x86_64 TLS (FS/GS base via MSR)

### Batch 5: Verification ✅
- [x] Verify `mmap` supports MAP_FIXED — Already implemented

### Deferred (not needed for basic Eyra)
- [ ] `fcntl` — F_GETFD/SETFD/GETFL/SETFL (add if needed)
- [ ] `tgkill` — Thread-directed signals (add if needed)
- [ ] `faccessat` — File access checks (add if needed)

## Files Modified

| File | Changes |
|------|---------|
| `crates/kernel/src/arch/aarch64/mod.rs` | Added 9 syscall numbers |
| `crates/kernel/src/arch/x86_64/mod.rs` | Added 10 syscall numbers (incl. ArchPrctl) |
| `crates/kernel/src/syscall/mod.rs` | Added dispatch for all new syscalls |
| `crates/kernel/src/syscall/process.rs` | Added gettid, exit_group, getuid/geteuid/getgid/getegid, arch_prctl |
| `crates/kernel/src/syscall/time.rs` | Added clock_getres |
| `crates/kernel/src/syscall/mm.rs` | Added madvise stub |
| `crates/kernel/src/syscall/sys.rs` | Added getrandom with PRNG |

## Progress Log

### 2026-01-09
- Implemented all Batch 1-4 syscalls
- Verified MAP_FIXED already supported
- All tests pass
- Build succeeds for both aarch64 and x86_64

## Handoff Notes

LevitateOS is now ready for basic Eyra integration testing. The following syscalls are implemented:

**New syscalls added:**
- `gettid` (178/186) — Returns thread ID
- `exit_group` (94/231) — Terminates process (same as exit for now)
- `getuid/geteuid/getgid/getegid` (174-177/102-108) — Return 0 (root)
- `clock_getres` (114/229) — Returns 1ns resolution
- `madvise` (233/28) — Stub, ignores advice
- `getrandom` (278/318) — PRNG seeded from timer
- `arch_prctl` (N/A/158) — x86_64 TLS via FS/GS MSRs

**If Eyra fails, check:**
1. Does it need `fcntl`? → Add F_GETFD/SETFD/GETFL/SETFL
2. Does it need `faccessat`? → Add file existence check
3. Does it need `tgkill`? → Add thread-directed signals

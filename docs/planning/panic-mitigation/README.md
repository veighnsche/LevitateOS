# Panic Mitigation Refactor Plan

**TEAM**: TEAM_415
**Created**: 2026-01-10
**Source**: TEAM_414 Panic Mitigation Checklist

---

## Overview

This refactor systematically eliminates unsafe panic paths in kernel-critical code. The goal is to ensure that malformed userspace input cannot crash the kernel.

---

## Phases

| Phase | Description | Priority | Status |
|-------|-------------|----------|--------|
| [Phase 1](phase-1.md) | Discovery and Safeguards | - | Ready |
| [Phase 2](phase-2.md) | Syscall Path Safety | P0 | Ready |
| [Phase 3](phase-3.md) | Filesystem Safety | P1 | Ready |
| [Phase 4](phase-4.md) | Task System Safety | P1 | Ready |
| [Phase 5](phase-5.md) | Cleanup and Hardening | P2 | Ready |

---

## Summary by Priority

### P0 - Critical (Syscall Paths)

**18 `unwrap()` calls** in syscall handlers that could crash the kernel on validation bugs:

- `process.rs` - getrusage, getrlimit
- `time.rs` - clock_gettime, gettimeofday, clock_getres
- `sys.rs` - getrandom
- `fs/*.rs` - fstat, pipe2, pread64, pwrite64, getcwd, statx, readv, writev

### P1 - High (API Safety)

- **`Tmpfs::root()`** - panics if called before init → should return `Option`
- **`current_task()`** - panics before scheduler init → should return `Option`

### P2 - Medium (Cleanup)

- Replace invariant `expect()` with `unsafe { unwrap_unchecked() }`
- Add `#[track_caller]` for better panic backtraces
- Fix `unimplemented!()` in x86_64

### Acceptable (No Action)

- Boot-time panics (system cannot continue)
- OOM handler (unrecoverable)
- CPU exception handlers (hardware faults)

---

## Execution Order

1. **Phase 1** - Verify baseline, document error handling strategy
2. **Phase 2** - Fix all syscall `unwrap()` calls (7 steps)
3. **Phase 3** - Fix `Tmpfs::root()` API (2 steps)
4. **Phase 4** - Fix `current_task()` API (2 steps)
5. **Phase 5** - Cleanup and hardening (3 steps)

---

## Success Criteria

- [ ] All syscall paths return proper error codes instead of panicking
- [ ] `Tmpfs::root()` returns `Option<Arc<Inode>>`
- [ ] `current_task()` returns `Option<Arc<Task>>`
- [ ] Build passes
- [ ] All behavior tests pass
- [ ] No dead code remaining

---

## References

- Source audit: `.teams/TEAM_414_panic_mitigation_checklist.md`
- Team file: `.teams/TEAM_415_refactor_panic_mitigation.md`

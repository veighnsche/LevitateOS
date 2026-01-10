# TEAM_417: Refactor Process Syscalls

**Created**: 2026-01-10
**Status**: Planned
**Plan**: `docs/planning/refactor-process-syscalls/`

---

## Objective

Refactor `crates/kernel/src/syscall/process.rs` from a 1090-line monolithic file into focused, single-responsibility modules under 500 lines each.

## Problem Statement

The current `process.rs` file:
- Exceeds the 1000-line hard limit (1090 lines)
- Mixes 8+ distinct functional areas
- Contains x86_64-specific code mixed with portable code
- Has scattered struct definitions and constants
- Is difficult to navigate and maintain

## Solution

Split into a `process/` module directory:

| Module | Purpose | Est. Lines |
|--------|---------|------------|
| mod.rs | Re-exports, Clone constants | ~50 |
| lifecycle.rs | exit, spawn, exec, waitpid | ~200 |
| identity.rs | uid/gid/tid, uname, umask | ~80 |
| groups.rs | pgid/sid syscalls | ~100 |
| thread.rs | clone, set_tid_address | ~150 |
| resources.rs | getrusage, prlimit64 | ~180 |
| arch_prctl.rs | x86_64 TLS handling | ~120 |

## Migration Strategy

1. No compatibility shims (Rule 5)
2. Extract one module at a time
3. Verify build after each extraction
4. Let compiler errors guide updates

## Behavioral Contracts

All public syscall signatures remain unchanged. No runtime behavior changes.

## Progress Log

- 2026-01-10: Plan created

## Verification

- [ ] Both architectures compile
- [ ] Behavior tests pass
- [ ] Runtime smoke test passes
- [ ] All files under 500 lines

# Question: Spawn Syscall Scope

**Team:** TEAM_118  
**Refactor:** Userspace Architecture  
**Created:** 2026-01-05

---

## Summary

Should the spawn syscall be implemented as part of the userspace refactor, or deferred to a separate phase?

---

## Context

The userspace refactor introduces an `init` process (PID 1). Without a spawn syscall:
- `init` can only print a banner and then "become" the shell (exec-style)
- It cannot respawn the shell if it exits
- There's no true process hierarchy

With a spawn syscall:
- `init` can spawn shell as a child process
- If shell exits, init can respawn it
- Proper process lifecycle management

---

## Options

### Option A: Skip spawn for now
- Init just transitions to shell (pseudo-exec)
- Simpler refactor
- Shell exit = system halt (current behavior)

### Option B: Implement spawn in this refactor
- Full process spawning
- Init can respawn shell
- More work, touches scheduler

### Option C: Defer spawn to Phase 8c (Recommended)
- Keep init minimal for now
- Refactor focuses on code organization
- Spawn implemented as dedicated follow-up

---

## Current Recommendation

**Option C** — Defer spawn to Phase 8c

Rationale:
1. This refactor is primarily about **code organization** (eliminating duplication)
2. Spawn syscall requires kernel scheduler integration
3. Keeping scope small reduces risk
4. Can still have a proper `init` crate that execs into shell

---

## Decision Required

Please confirm or choose an option:
- [ ] Option A: Skip spawn, init becomes shell
- [ ] Option B: Implement spawn in this refactor
- [x] Option C: Defer spawn to Phase 8c (recommended)

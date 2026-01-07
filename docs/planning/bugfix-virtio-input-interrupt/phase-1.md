# Phase 1: Understanding and Scoping

**Status**: ✅ Complete (from investigation)

## Bug Summary

| Aspect | Description |
|--------|-------------|
| Symptom | Ctrl+C doesn't interrupt processes |
| Severity | Medium - shell unusable for stopping runaway processes |
| Impact | User cannot interrupt blocked processes |

## Reproduction

**Steps:**
1. Boot LevitateOS
2. Run `signal_test`
3. When it reaches "Waiting for signal in pause()...", press Ctrl+C
4. Nothing happens - process stays blocked

**Expected:** Process should receive SIGINT and terminate/handle the signal  
**Actual:** No response to Ctrl+C

## Context

### Affected Code Areas

| File | Purpose |
|------|---------|
| `kernel/src/input.rs` | VirtIO input device driver (polling-based) |
| `kernel/src/syscall/fs/read.rs` | Only place that calls `input::poll()` |
| `kernel/src/syscall/signal.rs` | Signal delivery (`signal_foreground_process`) |
| `kernel/src/arch/aarch64/exceptions.rs` | IRQ handling |

### Recent Changes
- TEAM_240: Pipe EOF fix (unrelated)

## Constraints

- Must work with QEMU's VirtIO input device
- Must integrate with existing GIC interrupt handling
- Should not break existing keyboard input functionality

## Open Questions

None - root cause is clear from investigation.

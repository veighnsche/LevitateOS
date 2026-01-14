# TEAM_480: Refactor VM Control Separation

## Problem

VM control code was incorrectly placed in the `builder` crate (commits f42ef90, 477c40a, e3857a4). This violates separation of concerns:

- **builder**: Should ONLY build artifacts (kernel, initramfs, systemd, etc.)
- **xtask**: Should handle development tasks including VM control

The builder was controlling VMs, which is not its responsibility.

## Root Cause

Previous AI (Claude) conversations added VM control to builder without considering architecture. This is a recurring pattern where convenience trumps correctness.

## Fix Applied

1. Moved VM control modules from `crates/builder/src/builder/vm/` to `xtask/src/vm/`:
   - `commands.rs` - VM start/stop/send/status
   - `qmp.rs` - QEMU Machine Protocol client
   - `session.rs` - Session state management

2. Removed builder dependency from xtask

3. Removed VM commands from builder CLI

4. Made `qmp` module public in xtask for test access

## Files Changed

- `xtask/src/vm/mod.rs` - Rewrote to use local modules
- `xtask/src/vm/commands.rs` - NEW (moved from builder)
- `xtask/src/vm/qmp.rs` - NEW (moved from builder)
- `xtask/src/vm/session.rs` - NEW (moved from builder)
- `xtask/src/test/helpers.rs` - Updated imports
- `xtask/src/test/alpine.rs` - Updated imports
- `xtask/Cargo.toml` - Removed builder dependency
- `crates/builder/src/builder/mod.rs` - Removed vm module
- `crates/builder/src/main.rs` - Removed vm command handling
- `crates/builder/src/builder/vm/` - DELETED

## Timeout Improvements

Reduced unnecessary sleeps in VM commands:
- Start: 500ms → 200ms (just enough for socket creation)
- Stop: 500ms → 100ms (just enough for graceful shutdown signal)
- Send: Removed 100ms post-send sleep entirely
- QMP timeouts: 5s → 2s

## Architecture Reminder

```
builder/     → Builds things (kernel, initramfs, etc.)
xtask/       → Development tasks (VM control, tests, checks)
```

Never add runtime/VM control to builder again.

## Status

COMPLETE

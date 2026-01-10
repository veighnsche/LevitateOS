# Phase 1 — Discovery

## Feature Summary

**Problem:** Developers cannot easily interact with or inspect the running VM without full GDB attachment or manual QEMU monitor commands.

**Solution:** Add xtask commands that wrap QMP and serial I/O to provide ergonomic debugging tools.

**Who benefits:**
- Kernel developers debugging issues
- AI agents running automated tests
- Anyone investigating x86_64 GPU black screen (or similar issues)

---

## Success Criteria

| Criterion | Verification |
|-----------|--------------|
| Can execute arbitrary shell commands in VM | `cargo xtask shell exec "ls"` returns output |
| Can dump memory without stopping VM | `cargo xtask debug mem 0xffff8000` outputs hex |
| Can dump CPU registers via QMP | `cargo xtask debug regs` shows register values |
| Works for both aarch64 and x86_64 | Tested on both architectures |

---

## Current State Analysis

### What exists today:
1. **GDB support** (`cargo xtask run gdb --wait`) — Requires manual GDB connection
2. **QEMU Monitor** (`Ctrl+A C` in term mode) — Manual, not scriptable
3. **QMP client** (`xtask/src/support/qmp.rs`) — Low-level, only used for screenshots
4. **Serial I/O tests** — Pipe stdin to QEMU, but not exposed as a tool
5. **Test runner** — Runs pre-defined tests inside VM, no arbitrary commands

### Gaps:
- No way to run arbitrary commands from host → VM
- No QMP wrappers for memory/register inspection
- No unified "debug" subcommand

---

## Codebase Reconnaissance

### Modules to touch:
| Module | Change |
|--------|--------|
| `xtask/src/main.rs` | Add `Debug` and `Shell` subcommands |
| `xtask/src/support/qmp.rs` | Add `memsave`, `human-monitor-command` wrappers |
| `xtask/src/debug/` (NEW) | New module for debug commands |
| `xtask/src/shell/` (NEW) | New module for shell exec |

### Existing code to reuse:
- `QemuBuilder` — Start QEMU with correct args
- `QmpClient` — QMP socket communication
- Serial I/O pattern from `tests/serial_input.rs`

### Tests impacted:
- None directly (new functionality)
- May want to add tests for the new commands

---

## Constraints

1. **Non-invasive** — Debug commands should not require stopping the VM
2. **Headless-friendly** — Must work in `-nographic` mode for CI
3. **Timeout handling** — Commands must handle VM not responding
4. **Consistent UX** — Match existing xtask command patterns

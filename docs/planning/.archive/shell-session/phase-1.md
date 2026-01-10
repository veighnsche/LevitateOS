# Phase 1 — Discovery

## Feature Summary

**Problem:** The ephemeral `shell exec` command starts a fresh VM for every command (~10s boot time). For interactive debugging workflows, developers need a persistent VM session.

**Solution:** Add "session" subcommands that manage a long-lived QEMU process:
- User explicitly starts the VM
- Sends arbitrary text as keystrokes via QMP `sendkey`
- Can take screenshots
- User explicitly stops the VM

**Who benefits:**
- Developers debugging multi-step issues
- AI agents running complex test sequences
- Anyone needing visual + command interaction

---

## Success Criteria

| Criterion | Verification |
|-----------|--------------|
| Can start VM and keep it running | `shell start` → VM runs until stopped |
| Can send text to running VM | `shell send "ls"` → keystrokes appear in VM |
| Can take screenshot | `shell screenshot` → saves PNG |
| Can stop VM | `shell stop` → QEMU process terminated |
| State persists across commands | PID saved to file, QMP socket reused |

---

## Current State Analysis

### Existing tools:

| Tool | Description |
|------|-------------|
| `shell exec` (TEAM_323) | Ephemeral: starts VM, runs cmd, exits |
| `QmpClient` | Connects to QMP socket, executes commands |
| `screenshot.sh` | Starts VM, waits, takes screenshot, exits |

### Gaps:
- No command to start VM and **leave it running**
- No command to send keystrokes to **existing** VM
- No session state persistence (PID, socket path)

---

## Codebase Reconnaissance

### Modules to create/modify:

| Module | Change |
|--------|--------|
| `shell/mod.rs` | Add `start`, `send`, `screenshot`, `stop` commands |
| `shell/session.rs` (NEW) | Session management logic |
| `support/qmp.rs` | May need `sendkey` wrapper |

### QMP `sendkey` format:
```json
{"execute": "sendkey", "arguments": {"keys": [{"type": "qcode", "data": "a"}]}}
```

Each character must be translated to a QEMU keycode (qcode).

---

## Constraints

1. **One session at a time** — Only one VM can run per session
2. **State file** — Must persist PID and socket path to `.qemu-session.json`
3. **Clean cleanup** — `shell stop` must kill process and remove state file
4. **Keycode translation** — Must map ASCII → QEMU qcodes (including shift for uppercase)

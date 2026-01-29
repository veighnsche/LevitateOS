# TEAM_150: Shift+Tab VNC Keybinding Issue + ISO Filename Fix

**Date**: 2026-01-29
**Issue**: Shift+Tab keybinding not working in tmux through VNC
**Status**: RESOLVED - Changed to F2

## Additional Work: ISO/QCOW2 Filename Updates

Updated all references from `levitateos.iso` / `levitateos.qcow2` to `levitateos-x86_64.iso` / `levitateos-x86_64.qcow2` across the codebase.

**Files updated:**
- `.github/workflows/ci.yml` - CI artifact path
- `CLAUDE.md` - Visual testing documentation
- `RELEASE.md` - Release checklist
- `tools/recqemu/src/main.rs` - Help examples
- `tools/recqemu/README.md` - Usage examples
- `testing/install-tests/src/distro/levitate.rs` - Uses `distro_spec::levitate::ISO_FILENAME`
- `testing/install-tests/src/preflight.rs` - Uses `distro_spec::levitate::ISO_FILENAME`
- `testing/install-tests/README.md` - Documentation
- `testing/rootfs-tests/src/lib.rs` - Default qcow2 path
- `testing/rootfs-tests/src/preflight.rs` - Warning message
- `testing/rootfs-tests/tests/regression.rs` - Path documentation test
- `testing/rootfs-tests/CLAUDE.md` - Requirements
- `leviso/README.md` - Output artifacts table
- `leviso/docs/SUBSYSTEM_AUDIT.md` - Code examples
- `.teams/KNOWLEDGE_visual-install-testing.md` - All recqemu examples

## Problem

Shift+Tab keybinding for tmux pane switching does not work when accessing LevitateOS live ISO through VNC (TigerVNC).

### What We Tried

1. **BTab binding** - Standard tmux key name for Shift+Tab - **Did not work**
2. **S-Tab binding** - Alternative tmux notation - **Did not work**
3. **Tab binding** - Switched to plain Tab - **Did not work**
4. **C-o binding** - Added Ctrl+O as alternative - **Result unknown**

### Root Cause Analysis

Web search findings suggest this is a **VNC/terminal limitation**:
- VNC clients often send the same escape sequence for Tab and Shift+Tab
- Terminals cannot differentiate between these keys at the protocol level
- Different terminal emulators (TigerVNC, RealVNC, etc.) handle this inconsistently

**References**:
- [GitHub tmux issue #3181](https://github.com/tmux/tmux/issues/3181) - Ctrl+Tab/Shift+Tab not working in various terminals
- [tmux wiki: Modifier Keys](https://github.com/tmux/tmux/wiki/Modifier-Keys)

## Configuration

**File**: `leviso/profile/live-overlay/etc/profile.d/live-docs.sh`

Current binding:
```bash
bind-key -n BTab select-pane -t :.+
```

## Workarounds

1. **Use Ctrl+Left/Right** - Already implemented for pane resizing, could add for switching
2. **Configure VNC client** - Some VNC clients (PuTTY) allow configuring escape sequences
3. **Use different terminal** - Bypass VNC entirely (e.g., serial console, SSH)
4. **Try Ctrl+O** - Added but untested

## Resolution

Changed the primary keybinding from Shift+Tab (`BTab`) to **F2**:

1. Function keys are reliably passed through VNC (unlike modifier+key combinations)
2. F2 is memorable and doesn't conflict with shell operations
3. Kept `BTab` binding as fallback for native terminal users
4. Updated status bar and F1 help message to reflect new binding

**Files changed**: `leviso/profile/live-overlay/etc/profile.d/live-docs.sh`

## Notes

- BTab binding remains for users who access via native terminal/SSH
- F2 is the documented/advertised binding since it works everywhere
- This is a pragmatic solution to a VNC protocol limitation

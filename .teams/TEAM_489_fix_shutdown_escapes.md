# TEAM 489: Fix Shutdown/Reboot and Terminal Escape Issues

## Status: Complete

## Problems
1. Shutdown/reboot fail - missing `systemd-shutdown` binary
2. Terminal escape sequences (`[47;106R`) pollute user input

## Solution
1. Add `systemd-shutdown` to BINARIES list
2. Add kernel options to suppress escape sequences:
   - `systemd.log_color=false`
   - `vt.global_cursor_default=0`

## Files Changed
- `crates/builder/src/builder/fedora.rs` - Add systemd-shutdown
- `xtask/src/vm/commands.rs` - Kernel options
- `xtask/src/test/helpers.rs` - Kernel options for tests

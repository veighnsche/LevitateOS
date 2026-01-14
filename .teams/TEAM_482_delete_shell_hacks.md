# TEAM_482: Delete Shell Wrapper Hacks

## Summary

Deleted all shell-wrapper hacks that were bypassing proper shell functionality. Users now get real brush shell directly.

## What Was Deleted

### 1. `/bin/shell-wrapper`
A garbage script that ran each command in a subprocess via `brush -c "$cmd"`. Had no:
- Shell state (variables lost between commands)
- Job control
- History
- Tab completion
- Proper cd, ~, pipes

### 2. `/bin/brush-login`
Another wrapper script that was unused but polluting the system.

### 3. References in `/etc/shells` and `/etc/passwd`
Cleaned up to use `/bin/brush` directly.

## Why It Existed

TEAM_479 (Claude) created these hacks because brush was crashing on serial console with terminal query timeout. Instead of fixing brush properly, Claude patched around it with a broken wrapper.

This was wrong. The "workaround" was worse than the problem it solved.

## What's Fixed Now

```
Before: agetty → login → PAM → /bin/shell-wrapper (BROKEN)
After:  agetty → login → PAM → /bin/brush (REAL SHELL)
```

## Files Changed

- `crates/builder/src/builder/initramfs.rs` - Deleted shell-wrapper and brush-login generation
- `crates/builder/src/builder/auth/users.rs` - Changed user shells from `/bin/shell-wrapper` to `/bin/brush`

## Status

COMPLETE - Hacks deleted. If brush crashes on serial console, fix brush properly.

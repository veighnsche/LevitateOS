# TUI Autologin Debugging - Jan 29 2026

## Problem

The LevitateOS live ISO TUI (levitate-docs in tmux split pane) was working, then broke. Console autologin wasn't properly launching the interactive tmux session with documentation pane.

## Root Cause Analysis

The `console-autologin.service` (TTYPath=/dev/tty1) was starting but NOT attaching to the tmux session created by `live-docs.sh`. The session would be created in profile.d scripts, but the autologin shell couldn't attach to it.

## Changes Made

### 1. Fixed Console Autologin Script (`leviso/profile/live-overlay/usr/local/bin/autologin-shell`)

**Created new file** that sources `/etc/profile` to trigger profile.d scripts (including live-docs.sh):

```bash
#!/bin/bash
# Autologin wrapper that launches live environment

# Source profile to trigger live-docs.sh which creates and attaches to tmux session
# If live-docs.sh runs successfully, it will exec into tmux and this script won't continue
. /etc/profile

# Fallback to regular bash if live-docs.sh didn't exec (e.g., not on tty1, levitate-docs missing, etc.)
exec /bin/bash
```

**Why**: The systemd service needs an executable script to run on tty1. This script sources /etc/profile so that profile.d/live-docs.sh runs, which:
- Checks if on tty1 (skips for serial console)
- Checks if levitate-docs and tmux exist
- Creates a new tmux session named "live"
- Splits the window horizontally (shell on left, docs on right)
- Execs into the tmux session

### 2. Updated ISO Build (`leviso/src/component/custom/live.rs`)

**Added to `create_live_overlay_at()` function**:

```rust
// Include the wrapper script as a constant
const AUTOLOGIN_SHELL: &str = include_str!("../../../profile/live-overlay/usr/local/bin/autologin-shell");

// In create_live_overlay_at(), create the script with proper permissions:
let usr_local_bin = overlay_dir.join("usr/local/bin");
fs::create_dir_all(&usr_local_bin)?;
let autologin_path = usr_local_bin.join("autologin-shell");
fs::write(&autologin_path, AUTOLOGIN_SHELL)?;
fs::set_permissions(&autologin_path, fs::Permissions::from_mode(0o755))?;
```

**Why**: The wrapper script needs to be executable and included in the live overlay so console-autologin.service can run it.

### 3. Improved `recqemu vnc` Tool (`tools/recqemu/src/main.rs`)

**Added `--background` flag**:

```rust
/// Start in background and exit immediately (don't wait for QEMU)
#[arg(long)]
background: bool,
```

Changed execution logic to use `.spawn()` instead of `.status()` when backgrounding:

```rust
if background {
    // Spawn in background and return immediately
    builder.build_interactive().spawn().context("Failed to spawn QEMU")?;
    println!("QEMU started in background");
    Ok(())
} else {
    // Interactive mode: wait for QEMU to exit
    let status = builder.build_interactive().status()?;
    // ... cleanup ...
}
```

**Why**: The original `recqemu vnc --websockify &` would start QEMU but then hang in the background waiting for the process to exit, making it unsuitable for automated testing.

## Critical Issue: Build Caching

**GOTCHA**: When running `cargo run -- build iso` (rebuild ISO only), the FINAL phase components do NOT re-execute if the rootfs hasn't changed.

- `CustomOp::CopyDocsTui` (which builds and installs levitate-docs) is in the FINAL phase
- `cargo run -- build iso` only rebuilds the ISO from cached rootfs
- The FINAL phase only runs during full `cargo run -- build` (which rebuilds rootfs)

**To force rebuild of docs-tui**:
```bash
rm leviso/output/rootfs-staging* leviso/output/filesystem.erofs*
cargo run --manifest-path=leviso/Cargo.toml -- build
```

This will show:
```
Rebuilding levitate-docs...
 [3ms] compile  levitate-docs
Installed levitate-docs (101.1 MB)
Copying 5 library dependencies for levitate-docs...
```

## Current Status (Jan 29, 2026 - FIXED)

✅ **Working**:
- Console autologin executes on tty1
- /etc/profile is sourced
- live-docs.sh script runs
- tmux session "live" is created with split panes
- levitate-docs TUI displays on the RIGHT side
- Shell on the LEFT side
- recqemu vnc --background works properly
- System boots in ~20 seconds
- Welcome message displays

## Root Cause Analysis (Jan 29, 2026)

### Issue 1: Terminal Width
The original `split-window -h` (horizontal split, 50/50) gave the docs pane only 79 columns, but levitate-docs requires minimum 80 columns. The TUI would immediately exit with:
```
Error: Terminal too narrow (79 cols)
Minimum width: 80 columns
```

**Fix**: Use `split-window -h -l 88` to give docs pane exactly 88 columns (enough for 80 col minimum plus margin).

### Issue 2: Unicode Characters in Status Bar
The original status bar used Unicode arrows (←/→) which may cause issues with some terminals.

**Fix**: Changed to ASCII-safe `Left/Right` text.

## Final Working Script

```bash
exec tmux new-session -d -s live \; \
    set-option -g prefix None \; \
    set-option -g mouse on \; \
    set-option -g status-style 'bg=black,fg=white' \; \
    set-option -g status-left '' \; \
    set-option -g status-right ' Shift+Tab: switch | Ctrl+Left/Right: resize | F1: help ' \; \
    set-option -g status-right-length 60 \; \
    bind-key -n BTab select-pane -t :.+ \; \
    bind-key -n C-Left resize-pane -L 5 \; \
    bind-key -n C-Right resize-pane -R 5 \; \
    bind-key -n F1 display-message 'Shift+Tab: switch panes | Ctrl+Left/Right: resize | In docs: Up/Down navigate, j/k scroll, q quit' \; \
    split-window -h -l 88 levitate-docs \; \
    select-pane -t 0 \; \
    attach-session -t live
```

Key changes from broken version:
- `-l 88` instead of `-h` alone (fixed column width instead of percentage)
- ASCII-safe status bar text
- Simplified levitate-docs invocation (no error wrapper needed)

## Files Modified

1. `leviso/src/component/custom/live.rs` - Added autologin-shell creation
2. `leviso/profile/live-overlay/usr/local/bin/autologin-shell` - NEW FILE
3. `tools/recqemu/src/main.rs` - Added --background flag
4. Submodule updates: `tools/recqemu`

## Resolution Summary

The TUI autologin is now working. The docs-TUI appears on the RIGHT side of the screen as intended, with the shell on the LEFT. Users can:
- Use Shift+Tab to switch between panes
- Use Ctrl+Left/Right to resize panes
- Press F1 for help
- Navigate docs with arrow keys, j/k for scrolling, q to quit

## Testing Notes

**ALWAYS use 1920x1080 resolution for visual testing.** See CLAUDE.md "Visual Testing" section.

Previous debugging attempts used 1024x768 which is inappropriate for a daily-driver desktop OS.

## Related Issues

- TEAM_149: Installation test (where this debugging occurred)
- Original TUI fix that broke: See commit history before Jan 28 2026

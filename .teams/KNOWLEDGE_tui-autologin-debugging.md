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

## Current Status (Jan 29, 2026 - After Changes)

✅ **Working**:
- Console autologin executes on tty1
- /etc/profile is sourced
- live-docs.sh script runs
- tmux session "live" is created
- recqemu vnc --background works properly
- System boots in ~20 seconds
- Welcome message displays

❌ **Not Working**:
- tmux split-window isn't showing the levitate-docs pane
- Status bar shows only "0:bash" instead of "0:bash 1:levitate-docs"
- split-window command or levitate-docs binary execution appears to fail silently

## Unresolved Issues

### Why is split-window failing?

The tmux command in live-docs.sh:
```bash
exec tmux new-session -d -s live \; \
    ... config ... \
    split-window -h 'echo "Loading documentation..." && levitate-docs' \; \
    ... more config ... \
    attach-session -t live
```

Possible causes:
1. `split-window -h` command is failing within the tmux new-session chain
2. `levitate-docs` binary is crashing immediately after starting
3. `levitate-docs` binary is missing dependencies despite `copy_library()` calls
4. The complex tmux command chain has a syntax or ordering issue

### How to Debug

1. **Check if levitate-docs binary exists in live ISO**:
   ```bash
   # Mount the ISO and search for it
   mount -o loop leviso/output/levitateos.iso /mnt
   find /mnt -name "levitate-docs"
   ```

2. **Check if levitate-docs can run**:
   ```bash
   # Boot serial console and try running it
   recqemu serial leviso/output/levitateos.iso
   # At root prompt: /usr/bin/levitate-docs --help
   ```

3. **Check tmux manually**:
   ```bash
   # Boot into shell, then test tmux directly
   tmux new-session -d -s test
   tmux split-window -h -t test
   tmux list-windows -t test
   ```

## Files Modified

1. `leviso/src/component/custom/live.rs` - Added autologin-shell creation
2. `leviso/profile/live-overlay/usr/local/bin/autologin-shell` - NEW FILE
3. `tools/recqemu/src/main.rs` - Added --background flag
4. Submodule updates: `tools/recqemu`

## Next Steps

1. Debug why split-window fails (serial console or manual testing)
2. Check levitate-docs binary for runtime issues
3. Consider simpler approach if complex tmux setup is problematic
4. Add explicit error handling to live-docs.sh to log failures instead of silently returning

## Related Issues

- TEAM_149: Installation test (where this debugging occurred)
- Original TUI fix that broke: See commit history before Jan 28 2026

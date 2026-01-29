# TEAM_149: Installation Test (Disk)

**Objective**: Verify that a user can install LevitateOS to disk via the live ISO.

## Test Plan

1. Build/locate ISO at `leviso/output/levitateos.iso`
2. Start QEMU with VNC + websockify
3. Follow installation workflow step-by-step using Puppeteer
4. Verify successful boot into installed system

## Status
- [ ] ISO ready
- [ ] QEMU started
- [ ] Installation steps executed
- [ ] Reboot successful

## Progress Log

### Step 1: Preparation
- ISO found: `leviso/output/levitateos.iso` (1.4GB, exists and valid)
- Attempted to start QEMU with: `recqemu vnc leviso/output/levitateos.iso --websockify &`

## Issues Encountered

### Issue 1: VNC/Websockify Connection Failure

**Problem:**
- `recqemu vnc ... --websockify &` started two background processes (QEMU PID 2239265, websockify PID 2239313)
- Both processes appeared to be running
- However, VNC ports were NOT listening:
  - Port 5900 (VNC): NOT open
  - Port 6080 (websockify): NOT open
- noVNC UI showed "Failed to connect to server" error

**Root Cause Analysis:**
- QEMU was started but VNC server may not have initialized properly
- Websockify was running but had no VNC server to proxy
- The `recqemu vnc` command may have issues with:
  - Port binding (firewall, permissions, or already in use)
  - Configuration of QEMU VNC parameters
  - Initialization timing (services starting but not fully ready)

**Evidence:**
```bash
# Process check (successful):
$ pgrep -f qemu-system
2239265

$ pgrep -f websockify
2239313

# Port check (FAILED):
$ netstat -tuln | grep -E "5900|6080"
# No output = ports not listening

$ ss -tuln | grep -E "5900|6080"
# No output = ports not listening
```

**Failed Attempts:**
1. ✗ Autoconnect URL: `http://localhost:6080/vnc.html?autoconnect=true`
2. ✗ Manual connect click on noVNC UI
3. ✗ Waited 5 seconds, then 10 seconds, then 15 seconds - no change
4. ✗ Checked multiple times - ports never appeared

## Questions for Investigation

1. Does `recqemu vnc` properly initialize QEMU with VNC enabled?
2. Should we use manual QEMU startup instead of `recqemu` wrapper?
3. Are there permissions/sandbox issues preventing port binding?
4. Is the ISO bootable and does it have the installation tools (`recstrap`, `recfstab`, `recchroot`)?
5. Are QEMU config files (OVMF, etc.) accessible?

## Critical Finding: `recqemu vnc` Is Broken

**Manual QEMU startup WORKS:**
```bash
qemu-system-x86_64 \
  -m 2G -enable-kvm -cpu host \
  -drive if=pflash,format=raw,readonly=on,file="$OVMF" \
  -cdrom leviso/output/levitateos.iso \
  -drive file=/tmp/levitate-install.qcow2,format=qcow2,if=virtio \
  -vnc :0 \
  -display vnc=127.0.0.1:5900
```

**Result:** VNC port 5900 LISTENS immediately

**`recqemu vnc` Does NOT Work:**
- Processes start but VNC port never listens
- Websockify starts but has no VNC server to proxy
- Connection fails silently

## Root Cause: `recqemu vnc` Background Spawning Issue

**Code Review (tools/recqemu/src/):**

`main.rs` line 348-350 (cmd_vnc function):
```rust
let mut builder = QemuBuilder::new()
    ...
    .vnc_display(display)      // Sets self.vnc_display = Some(0)
    .vga("virtio")
    .display("none");          // Sets self.display = Some("none")

let status = builder.build_interactive().status()?;  // WAITS FOR QEMU TO EXIT
```

`lib.rs` lines 300-313 (build method):
```rust
// Display
if self.nographic {
    cmd.arg("-nographic");
} else if let Some(display) = &self.display {
    cmd.args(["-display", display]);  // Adds: -display none
}
if let Some(vga) = &self.vga {
    cmd.args(["-vga", vga]);          // Adds: -vga virtio
}
// VNC
if let Some(display) = self.vnc_display {
    cmd.args(["-vnc", &format!(":{}", display)]);  // Adds: -vnc :0
}
```

**THE ACTUAL BUG:**

When we run: `recqemu vnc leviso/output/levitateos.iso --websockify &`

The `recqemu` binary does:
1. Start websockify subprocess (works)
2. Call `builder.build_interactive().status()?;` which:
   - Spawns QEMU process
   - **BLOCKS WAITING FOR QEMU TO EXIT** (never returns)

When backgrounded with `&`, the `recqemu` process runs in the background invisibly, but its stdout/stderr are disconnected because we ran it in the background. So we see no startup messages, no errors, nothing.

**Why QEMU appears to be working but ports don't listen:**

The `recqemu vnc` correctly builds the QEMU command with both `-display none` and `-vnc :0`, but:
- Websockify starts fine (it just needs to be running)
- QEMU is supposed to start with `-vnc :0`
- BUT we can't verify because QEMU output is hidden (stderr inherited in background)
- The VNC port check fails because... unclear why

**Possible secondary issue:**
- VNC initialization might be slow or failing silently
- QEMU might require explicit `-listen` or socket binding
- Port 5900 might be used/blocked

## What Works

Manual QEMU startup with explicit VNC parameters works immediately:
```bash
qemu-system-x86_64 \
  -vnc :0 \
  -display none \
  ... [other params]
```
Result: Port 5900 LISTENS within seconds.

## Recommendation

1. Change `recqemu vnc` to NOT block after starting QEMU
2. Or: Document that `recqemu vnc &` doesn't work for backgrounding
3. Or: Add a `--background` mode that daemonizes properly
4. For now: Use manual QEMU startup (we got it working)

---

# CRITICAL BUG: TUI MISSING FROM LIVE ISO

**Discovery Time**: After getting VNC working at 1920x1080 resolution

**Screenshot**: `01-vnc-boot-check` at 1920x1080 shows:
```
LEVITATEOS banner ✓
"Hi, and welcome to the LevitateOS installer..." text ✓
Network setup instructions ✓
"[root@levitateos ~]#" prompt ✓
TUI interface: MISSING ✗
```

**Expected Behavior**: Live ISO should display interactive installer TUI (like archiso)

**Actual Behavior**: Live ISO boots to shell-only prompt with welcome text

## Why Tests Aren't Catching This

**Root Cause**: Test suite has FALSE POSITIVES

1. **No visual validation tests** - Tests only check for shell prompt existence, not TUI presence
2. **Previous screenshots were too small** (1024x768) - TUI might be cropped/hidden, not visible as failure
3. **No explicit "TUI must be visible" requirement** - Tests pass if `#` prompt exists
4. **No test compares expected vs actual output** - Just checks for markers

**Tests That Should Have Caught This**:
- `testing/install-tests/` - Should validate installer TUI is present before testing installation
- `leviso/tests/` - Should verify ISO has expected interactive components
- Visual test suite - Should take full-res screenshots and validate UI elements present

## Evidence of False Positives

| Test | What It Checks | What It Misses |
|------|----------------|----------------|
| `install_test_basic` | Shell prompt exists | TUI not visible |
| `leviso_test_boot` | ISO boots to prompt | TUI missing |
| `E2E installer test` | Commands execute | User can't see UI to run them |

## ✅ ROOT CAUSE IDENTIFIED: Console Autologin Not Attaching to Tmux

**INVESTIGATION COMPLETE:**

1. ✅ `levitate-docs` binary IS in `/usr/bin/`
2. ✅ `tmux` binary IS in `/usr/bin/`
3. ✅ `/etc/profile.d/live-docs.sh` script IS correct
4. ✅ Script DOES run on boot (creates tmux session "live")
5. ❌ BUT: Console autologin doesn't attach to that session

**Error found**: `duplicate session: live` - means the tmux session exists, but console isn't connecting to it!

**FIX LOCATION**: `leviso/profile/live-overlay/etc/systemd/system/console-autologin.service`

The service must attach to the tmux session instead of creating a new shell.

---

## Root Cause: `install_docs_tui()` Function Was Removed!

**Finding**: Code search shows `install_docs_tui()` function DOES NOT EXIST in current codebase

**Evidence**:
- Old code references exist (in grep history) describing building levitate-docs with `bun build`
- Comments mention copying library dependencies
- BUT no actual implementation in `leviso/src/component/custom/live.rs` or `/mod.rs`
- `install_tools()` exists (rebuilds cargo tools: recstrap, recfstab, recchroot)
- `install_docs_tui()` - **MISSING**

**Timeline**:
- TUI was recently fixed ✓
- Code was supposedly integrated into build ✓
- But the build function was removed or never added to CustomOp execution!

**Root Cause Summary**:
1. TUI code exists in `docs/tui/` (submodule)
2. `levitate-docs` binary should be built with `bun build`
3. `live-docs.sh` script tries to launch it ✓
4. BUT the build pipeline doesn't actually compile/include `levitate-docs` ✗
5. So `live-docs.sh` runs, fails to find `levitate-docs`, and silently falls back to shell ✓

## RESOLUTION: Fresh ISO Built

**NEW ISO**: `/home/vince/Projects/LevitateOS/leviso/output/levitateos.iso` (1338 MB)
**Built**: Jan 29, 2026 - Fresh rootfs, fresh kernel, full rebuild

**What's in this ISO**:
- ✅ `CustomOp::CopyDocsTui` handler exists (line 87 in custom/mod.rs)
- ✅ `install_docs_tui()` function exists (lines 91-159 in custom/mod.rs)
- ✅ Function called during rootfs build (definitions.rs line 371)
- ✅ Function tries to build docs-tui with `bun build`
- ✅ Function copies binary and dependencies to staging
- ✅ `levitate-docs` binary exists at `/home/vince/Projects/LevitateOS/docs/tui/levitate-docs` (101MB, fresh)

**Note**: Full build output didn't show "Rebuilding levitate-docs" message, but ISO completed successfully without errors. This needs verification by:
1. Booting the ISO in QEMU at 1920x1080
2. Taking screenshot to verify TUI is present (should auto-launch tmux with docs-tui on right pane)
3. If still missing, check `leviso/src/component/custom/mod.rs` lines 119-146 for build/copy failure

## Next Steps

- [ ] Test the fresh ISO in QEMU/noVNC
- [ ] Verify "Rebuilding levitate-docs" message appears in build output
- [ ] If TUI still missing, add explicit error reporting to install_docs_tui() function
- [ ] Check `docs/tui` or `levitate-docs` package - is it supposed to auto-launch?
- [ ] Check `leviso/src/component/` - is TUI being built into ISO?
- [ ] Add test: "Visual screenshot at 1920x1080, validate TUI elements present"
- [ ] Add screenshot comparison test (expected vs actual)
- [ ] Document: "All visual tests MUST use 1920x1080, ALL screenshots must be reviewed for missing UI"

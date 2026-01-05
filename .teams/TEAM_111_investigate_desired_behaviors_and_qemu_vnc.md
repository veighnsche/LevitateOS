# TEAM_111: Investigate Desired Behaviors + QEMU VNC for Visual Debugging

## 1. Pre-Investigation Checklist

### 1.1 Team Registration
- **Team ID:** TEAM_111
- **Team File:** `.teams/TEAM_111_investigate_desired_behaviors_and_qemu_vnc.md`

### 1.2 Context
- **User Request:** Thorough investigation of what the ACTUAL desired stable behaviors should be
- **End Goal:** Terminal visible on QEMU screen
- **Blocker:** User cannot see QEMU screen; needs browser-based VNC

### 1.3 Previous Teams Referenced
- TEAM_109: Fix GPU driver - documented VirtQueue issues
- TEAM_110: GPU fallback investigation - claims GPU is "fixed"

---

## 2. Phase 1: Investigation Findings

### 2.1 Current State Summary

**What Works (via Serial):**
- Full boot sequence completes
- All VirtIO devices initialize (GPU, Block, Net, Input)
- Shell launches and shows `# ` prompt
- Kernel reaches "steady state"

**What's Unclear (need visual verification):**
- Does the GPU actually display content in QEMU window?
- Is the QEMU window showing "Display output is not active"?
- Can we see terminal text rendered on the GPU framebuffer?

### 2.2 Behavior Test Analysis

**Current Golden Log (`tests/golden_boot.txt`):**
- 37 lines, ends at `[TICK] count=0`

**Actual Boot Output (`tests/actual_boot.txt`):**
- 53 lines, includes shell startup and prompt
- Ends at `# ` (shell prompt)

**Delta:**
```
Lines 38-53 are NEW and include:
- [SPAWN] Looking for 'hello' in initramfs...
- ELF loading messages
- User process creation
- Shell startup banner
- # prompt
```

### 2.3 The False Positive Problem

Previous teams documented that `levitate-gpu` (virtio-drivers) gives FALSE POSITIVE tests:
- Driver initializes without error
- Tests pass because they check init, not display output
- QEMU window shows nothing / "Display output is not active"

**TEAM_110 claims** they fixed this by:
- Removing `levitate-gpu`
- Using `levitate-drivers-gpu` which has `SET_SCANOUT` and `RESOURCE_FLUSH`

**HOWEVER:** This cannot be verified without seeing the QEMU display!

---

## 3. The Desired Baseline: What Should Actually Happen

### 3.1 The End Goal
**A visible terminal on the QEMU screen** with:
- Text rendering (LevitateOS shell prompt)
- Command echo (typing shows characters)
- Scrolling (text moves up when terminal fills)

### 3.2 Required VirtIO GPU Behavior
1. `GET_DISPLAY_INFO` - Get resolution (1280x800)
2. `RESOURCE_CREATE_2D` - Create framebuffer resource
3. `RESOURCE_ATTACH_BACKING` - Attach memory to resource
4. `SET_SCANOUT` - Configure scanout to display resource
5. `TRANSFER_TO_HOST_2D` - Copy pixel data to host
6. `RESOURCE_FLUSH` - Display to screen

### 3.3 Proposed Golden Behavior Checklist

| Step | Serial Output | GPU Action |
|------|---------------|------------|
| Stage 1 | `[BOOT] Stage 1: Early HAL` | None |
| Stage 2 | `MMU re-initialized...` | None |
| Stage 3 | `GPU initialized successfully` | Framebuffer visible |
| Stage 3 | `[TERM] Terminal::new` | Text rendering ready |
| Stage 4 | `VirtIO Block/Net/Input` | None |
| Stage 5 | `LevitateOS Shell` | Shell prompt visible |

---

## 4. Plan: Set Up QEMU VNC in Browser

### 4.1 Technical Approach

QEMU supports VNC natively with WebSocket. We can:
1. Run QEMU with `-vnc :0,websocket=5700`
2. Run noVNC (HTML5 VNC client) to connect
3. View QEMU display in browser

### 4.2 Implementation Steps

#### Step 1: Create run-vnc.sh script
```bash
#!/bin/bash
# Run QEMU with VNC enabled for browser viewing

cargo build -p levitate-kernel --release --target aarch64-unknown-none --features verbose
aarch64-linux-gnu-objcopy -O binary target/aarch64-unknown-none/release/levitate-kernel kernel64_rust.bin

qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -m 1G \
    -kernel kernel64_rust.bin \
    -display none \
    -vnc :0,websocket=5700 \
    -device virtio-gpu-device,xres=1280,yres=800 \
    -device virtio-keyboard-device \
    -device virtio-tablet-device \
    -device virtio-net-device,netdev=net0 \
    -netdev user,id=net0 \
    -drive file=tinyos_disk.img,format=raw,if=none,id=hd0 \
    -device virtio-blk-device,drive=hd0 \
    -initrd initramfs.cpio \
    -serial mon:stdio \
    -no-reboot
```

#### Step 2: Start noVNC
```bash
# Install noVNC if not present
sudo apt install novnc websockify

# Run noVNC (will proxy WebSocket to VNC)
novnc --listen 6080 --vnc localhost:5700
```

#### Step 3: Open in browser
Navigate to: `http://localhost:6080/vnc.html`

### 4.3 Alternative: Use xtask for VNC mode
Create `cargo xtask run-vnc` command that:
1. Starts QEMU with VNC
2. Launches noVNC in background
3. Prints URL to open

---

## 5. Proposed Test Updates

### 5.1 Update Golden Log
The golden log should reflect the ACTUAL desired behavior (full boot to shell):

```
[BOOT] Stage 1: Early HAL (SEC)
Heap initialized.
[BOOT] Stage 2: Memory & MMU (PEI)
MMU re-initialized (Higher-Half + Identity).
... (existing stages) ...
[BOOT] Starting shell from initramfs...
[TICK] count=0
[SPAWN] Looking for 'hello' in initramfs...
... (ELF loading) ...
LevitateOS Shell (lsh) v0.1
Type 'help' for commands.

# 
```

### 5.2 Add Visual Verification Test
Create `cargo xtask test gpu-visual` that:
1. Runs QEMU with GPU
2. Captures framebuffer via QMP
3. Verifies non-blank pixels (not all black/white)

---

## 6. Next Steps (User Decision Required)

1. **Immediate:** Implement VNC setup so we can SEE the QEMU display
2. **Then:** Verify if GPU actually works or shows blank
3. **Finally:** Either:
   - Update golden log if GPU works
   - Debug GPU driver if display is blank

---

## 7. Session Checklist

- [x] Team file created
- [x] Investigation completed
- [x] Current vs expected behavior documented
- [x] VNC setup implemented (`run-vnc.sh` + `cargo xtask run-vnc`)
- [x] Visual verification performed: **GPU is BROKEN**
- [x] xtask robustness fixes applied
- [x] Future team handoff documented
- [x] Tests updated for desired baseline (golden log + behavior checks)
- [ ] GPU driver fixed (for next team)
- [x] Golden log updated (to include shell prompt)

## 8. Added Tooling

### `cargo xtask run-vnc`

AI agents can use this command to verify GPU display:

```bash
cargo xtask run-vnc
```

Then in browser:
1. Navigate to `http://localhost:6080/vnc.html`
2. Click "Connect"
3. Check display:
   - "Display output is not active" = GPU BROKEN ❌
   - Terminal text visible = GPU WORKING ✅

---

## 9. xtask Robustness Investigation

### Issues Found & Fixed

| Issue | Fix |
|-------|-----|
| Hardcoded websockify path (`~/.local/bin`) | `find_websockify()` searches PATH, pip, pipx locations |
| No install instructions for missing deps | Clear error message with dnf/apt/pip commands |
| `pkill -f qemu` too broad | Specific pattern `qemu.*-vnc.*:0` to avoid killing other QEMU |
| No health check after websockify spawn | `try_wait()` verifies process started |
| Idempotency unclear | Documented; kills existing processes before starting |

### Idempotency: ✅ YES

The command is idempotent:
- Kills existing websockify on port 6080
- Kills existing QEMU with VNC :0
- Downloads noVNC only if not present
- Safe to run multiple times

### Test Results

- ✅ Compiles without errors
- ✅ websockify detection works
- ✅ QEMU VNC starts on port 5900
- ✅ noVNC accessible on port 6080
- ✅ Browser can connect and view display

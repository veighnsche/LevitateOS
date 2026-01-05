---
description: how to verify GPU display state when QEMU window is blank
---

# Verifying GPU State via QMP

When the QEMU graphical window reports "Display output is not active", follow these steps to verify if the kernel is actually rendering.

## 1. Prerequisites
- QEMU must be running with the QMP socket enabled (enabled by default in `run.sh` via `TEAM_092`).
- The `qmp.sock` file should be present in the project root.

## 2. Capture a Framebuffer Dump
Run the following command from the project root:

```bash
cargo xtask gpu-dump screenshot.png
```

## 3. Analyze the Results
- **If `screenshot.png` has contents (e.g., text, terminal):** The kernel-side driver is WORKING. The issue is likely a QEMU surface timing issue or a "heartbeat" flush rate problem.
- **If `screenshot.png` is all black/red:** The kernel is not correctly rendering into the framebuffer, or the VirtIO GPU scanout is not pointing to the correct memory.

## 4. Check Internal Heartbeats
Monitor the serial console for `[GPU-HB]` logs. 
- These report `total_flushes` and `failed_flushes`.
- If `failed_flushes` is increasing, the VirtIO queue is entering an error state.

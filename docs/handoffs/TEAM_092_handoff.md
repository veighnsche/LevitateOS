# TEAM_092 Handoff: GPU & Terminal Modularization

## 📋 Overview
TEAM_092 has completed the extraction of the GPU and Terminal subsystems into independent library crates. This refactor addresses technical debt, simplifies the kernel core, and provides robust host-side verification tools.

---

## 🏗️ New Architectural Patterns

### 1. Library Extraction (Separation of Mechanism and Policy)
Hardware logic is now strictly separated from emulation logic:
- **`levitate-gpu`**: The *mechanism* for VirtIO GPU interaction. Implements `DrawTarget` for generic graphical output.
- **`levitate-terminal`**: The *policy* for ANSI emulation and text rendering. Platform-agnostic; rendors to any `DrawTarget`.
- **`levitate-hal`**: Now hosts `VirtioHal` and `StaticMmioTransport`, providing a unified foundation for all VirtIO drivers.

**Recommendation:** Future drivers (Block, Net, Input) should follow this pattern by extracting their core logic into `levitate-*` crates.

### 2. DrawTarget Abstraction
Using `embedded-graphics` traits (`DrawTarget`, `OriginDimensions`) allows us to swap rendering targets easily. The Terminal no longer knows about the GPU; it only knows about a `DrawTarget`.

---

## ⚠️ Critical Gotchas & Bug Patterns

### 1. Recursive Deadlocks in IRQ Context
**Finding:** Using `serial_println!` inside a timer-based telemetry hook can deadlock if the shell is currently printing.
**Workaround:** ALWAYS use `console::WRITER.try_lock()` in interrupt handlers or high-frequency telemetry hooks.
**Example:**
```rust
if let Some(mut uart) = levitate_hal::console::WRITER.try_lock() {
    let _ = writeln!(uart, "Metric: {}", value);
}
```

### 2. Mutable Borrow Collisions with Framebuffers
**Finding:** Capturing `self.state.width` inside a loop while `self.state.framebuffer()` (which returns `&mut [u8]`) is active causes a borrow-checker conflict.
**Pattern:** Capture all required fields *before* calling the mutable buffer accessor.

---

## 🔍 Verification Techniques

### 1. `xtask gpu-dump` (Ground Truth)
If the QEMU window shows "Display output not active," do not assume the driver is failing.
- **Command:** `cargo xtask gpu-dump snapshot.png`
- **What it does:** Uses QMP to dump the guest's VRAM directly from the host.
- **Why use it:** Verifies that pixels are being correctly written to the guest framebuffer, isolating display scanout issues from rendering logic.

### 2. GPU Heartbeat (`[GPU-HB]`)
The kernel now outputs a heartbeat at 1Hz on serial (if verbose mode enabled).
- **Format:** `[GPU-HB] <width>x<height> | flushes: <count> | errors: <count>`
- **Usage:** Confirms that the GPU task is alive and the driver is attempting flushes.

---

## 🚀 Future Work (Finalizing the Refactor)
While rendering is verified, the QEMU graphical display remains blank on some hosts. 
- **Next Goal:** Investigate VirtIO GPU Scanout configuration (RESOURCE_SET_SCANOUT).
- **Status:** Rendering logic is proven correct via `gpu-dump`; only the host-side presentation is pending.

---

## 🔗 Traceability
- **Team Log:** `.teams/TEAM_092_refactor_gpu_observability.md`
- **Main Progress:** `docs/ROADMAP.md` (Phase 2 complete)
- **Gotchas:** `docs/GOTCHAS.md` (Updated items 4, 7)

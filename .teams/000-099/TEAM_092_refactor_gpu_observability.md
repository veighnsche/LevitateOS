# TEAM_092: GPU & Console Telemetry Refactor

## 1. Pre-Planning Checklist

### 1.1 Team Registration
- **Team ID:** TEAM_092
- **Refactor Summary:** Decouple the VirtIO GPU hardware abstraction from Terminal emulation and implement robust "peek/poke" debugging tools via `xtask` and QEMU's monitor.

### 1.2 Refactor Intent
- **Pain Points:** 
    - Silent GPU failures ("Display output is not active").
    - Tight coupling between character rendering and hardware flushes.
    - Lack of visibility into the guest's framebuffer state from the host.
    - Non-deterministic behavior test failures (golden logs vs actual output).
- **Success Criteria:**
    - `cargo xtask gpu-dump`: A new command to capture the guest's screen and report VirtIO state.
    - Decoupled `Terminal` and `GpuController` with a clear "health check" interface.
    - Kernel-side "Telemetry Port": A way to stream GPU hardware status to the serial console on intervals.
    - Automated QEMU tracing integration.

### 1.3 Context Check
- [x] Read Project Overview (Unix-Rust Philosophy)
- [x] Read current active phase (Multitasking / Userspace)
- [x] Scan recent team logs (TEAM_091 fixed the hang, but observability is still zero)
- [x] Run test suite (Verified: Behavior tests are failing line counts, but kernel boots)

---

## 2. Planning Structure

- [x] Phase 1: Discovery & Telemetry Hooks
## Handoff
- **Handoff Document:** [TEAM_092_handoff.md](file:///home/vince/Projects/LevitateOS/docs/handoffs/TEAM_092_handoff.md)
- **Status:** Phase 2 complete. Telemetry active. Modular architecture verified.
- [x] Phase 2: Structural Extraction (Decoupling)
- [ ] Phase 3: xtask "Peeking" Tools (Complete: gpu-dump)
- [ ] Phase 4: Self-Healing & Integration
- [ ] Phase 5: Hardening & Handoff

## Key Findings (TEAM_092)
1. **Recursive Deadlock**: Serial output during IRQ handlers must use `try_lock()`.
2. **Modular Libraries**: Decoupled `levitate-gpu` and `levitate-terminal` successfully.
3. **Observability**: `xtask gpu-dump` provides reliable ground truth for display issues.

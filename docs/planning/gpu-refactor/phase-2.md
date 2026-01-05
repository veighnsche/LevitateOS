# Refactor Phase 2 — Structural Extraction

## Target Design
We will separate the "Policy" of display updates from the "Mechanism" of pixel writing.

### New Components:
- **`GpuHardware` Trait:** Defines low-level VirtIO operations (transfer, flush).
- **`Framebuffer` Struct:** Manages a memory backing and tracks "dirty" regions.
- **`TerminalEmulator`:** A pure logic component that writes characters to a `Framebuffer`.
- **`DisplayManager`:** The orchestrator that decides when to sync the `Framebuffer` to the `GpuHardware`.

## Extraction Strategy
1. Extract `Terminal` logic from [terminal.rs](file:///home/vince/Projects/LevitateOS/kernel/src/terminal.rs) into a backend-agnostic struct.
2. Introduce a `DirtyRect` tracking mechanism in the `Framebuffer` to allow partial updates (Rule 20: Efficiency).
3. Move the global `GPU` lock to a more granular `GpuController` that doesn't block the `Terminal` logic.

## Steps
1. **Step 1:** Define `GpuHardware` and `Framebuffer` traits.
2. **Step 2:** Refactor `Terminal` to use a `DrawingContext` instead of a direct `GpuState` reference.
3. **Step 3:** Implement an asynchronous `FlushService` that runs on the timer.

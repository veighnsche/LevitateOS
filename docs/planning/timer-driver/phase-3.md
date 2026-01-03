# Phase 3 — Implementation: AArch64 Generic Timer Driver

## Implementation Overview
This phase builds the `Timer` driver in `levitate-hal` and integrates it into the kernel.

## Design Reference
- [Phase 2 — Design](file:///home/vince/Projects/LevitateOS/docs/planning/timer-driver/phase-2.md)
- [Implementation Plan](file:///home/vince/.gemini/antigravity/brain/a273405f-0bdf-4448-87ce-39dda8d6dc28/implementation_plan.md)

## Steps

### Step 1 – Add Register Control with bitflags
- [ ] Add `bitflags` dependency to `levitate-hal/Cargo.toml`.
- [ ] Implement `TimerCtrlFlags` in `levitate-hal/src/timer.rs`.

### Step 2 – Implement Core Timer Logic
- [ ] Implement `Timer` trait and `AArch64Timer` struct.
- [ ] Add `read_counter`, `read_frequency`, `set_timeout`, and `configure` using AArch64 system registers.

### Step 3 – Integrate and Cleanup
- [x] Export `timer` from `levitate-hal/src/lib.rs`.
- [x] Update `kernel/src/main.rs` to use the new API.
- [x] Remove `kernel/src/timer.rs`.

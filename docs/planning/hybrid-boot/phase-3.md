# Phase 3: Implementation — Hybrid Boot Specification

## Implementation Overview
Refactor `kernel/src/main.rs` and `kernel/src/terminal.rs` to enforce the Boot State Machine.

## Design Reference
- See [Phase 2 Design](file:///home/vince/Projects/LevitateOS/docs/planning/hybrid-boot/phase-2.md) for enum definitions and behavioral contracts.

## Steps and Units of Work

### Step 1: Enforce Architectural Boundaries
- **UoW 1**: Move memory reservation logic entirely into Stage 2. (TEAM_061)
- **UoW 2**: Decouple UART logging from the main loop so it functions from Stage 1.

### Step 2: Interactive Terminal Refinement
- **UoW 1**: Implement `0x08` (Backspace) and `0x0D/0x0A` (Newline) destructive logic in `terminal.rs`. (COMPLETED by TEAM_060 - verify in Phase 4).
- **UoW 2**: Implement tab-stop logic (SPEC-4).

### Step 3: Stage Transitions and Logging
- **UoW 1**: Add explicit `[BOOT] Stage X` UART logs (COMPLETED by TEAM_061 - verify in Phase 4).
- **UoW 2**: Ensure "SYSTEM_READY" is the final transition.

# UoW 1: Boot State Machine Definition
**Parent Step**: Phase 2 - Step 1
**Parent Phase**: Phase 2 - Design

## Goal
Define the `BootStage` enum and the transition logic (entry/exit checks) to enforce the hybrid boot flow.

## Context
Currently, `main.rs` uses comments to demarcate stages. We want to move to a type-safe state machine.

## Tasks
- [ ] Define `enum BootStage` with variants: `EarlyHAL`, `MemoryMMU`, `BootConsole`, `Discovery`, `SteadyState`.
- [ ] Define the `transition_to(stage: BootStage)` helper that logs to UART.
- [ ] Specify error handling (Panic vs. Maintenance Shell) for stage failures.

## Expected Outputs
- Code snippet or prototype for the `BootStage` state machine.

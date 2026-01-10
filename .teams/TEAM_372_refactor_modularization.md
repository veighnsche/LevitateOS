# Team 372: Modularization Refactor

- **ID**: TEAM_372
- **Objective**: Refactor oversized modules and reorganize the kernel crate for better maintainability (Rule 7).
- **Refactor Plan Root**: `docs/planning/modularization-refactor/`

## Scope
- Split [gic.rs](file:///home/vince/Projects/LevitateOS/crates/hal/src/aarch64/gic.rs) into a modular structure.
- Split [init.rs](file:///home/vince/Projects/LevitateOS/crates/kernel/src/init.rs) into functional submodules.
- Split [process.rs](file:///home/vince/Projects/LevitateOS/crates/kernel/src/syscall/process.rs) into functional submodules.
- Clean up [arch/x86_64/mod.rs](file:///home/vince/Projects/LevitateOS/crates/kernel/src/arch/x86_64/mod.rs) by extracting types and syscall numbers.
- Reorganize [kernel/src](file:///home/vince/Projects/LevitateOS/crates/kernel/src) to move subsystem wrappers to a `subsystems/` directory.

## Status
- [ ] Phase 1: Discovery and Safeguards
- [ ] Phase 2: Structural Extraction
- [ ] Phase 3: Migration
- [ ] Phase 4: Cleanup
- [ ] Phase 5: Hardening and Handoff

## Log
- **2026-01-10**: Registered team and initiated refactor planning under `make-a-refactor-plan` workflow.

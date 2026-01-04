# UoW 1: Industry Alignment (UEFI & Linux)
**Parent Step**: Phase 1 - Step 1
**Parent Phase**: Phase 1 - Discovery

## Goal
Map UEFI Platform Initialization (PI) phases and Linux `start_kernel` milestones to LevitateOS stages to ensure architectural consistency.

## Context
LevitateOS currently uses 5 stages in `kmain`. We need to verify these against:
1. **UEFI SEC/PEI/DXE/BDS** phases.
2. **Linux `setup_arch` to `rest_init`** flow.

## Tasks
- [ ] Review UEFI PI Spec (v2.9+) for SEC/PEI transition details.
- [ ] Review Linux `init/main.c` for `start_kernel()` initialization order.
- [ ] Draft the comparative table in `docs/BOOT_SPECIFICATION.md`. (DONE by TEAM_060 - verifying accuracy).

## Expected Outputs
- A confirmed mapping table that justifies the 5-stage split.

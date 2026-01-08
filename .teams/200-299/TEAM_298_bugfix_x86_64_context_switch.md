# TEAM_298: Bugfix - x86_64 Context Switch Corruption

**Previous Investigation**: [.teams/TEAM_298_investigate_x86_64_context_switch.md](.teams/TEAM_298_investigate_x86_64_context_switch.md)
**Plan Directory**: [.plans/x86_64_context_switch_fix/](.plans/x86_64_context_switch_fix/)

## Status
- **Phase 1 (Understanding)**: Complete.
- **Phase 2 (Root Cause)**: Complete (Identified RFLAGS missing & Global state issues).
- **Phase 3 (Design)**: In Progress.

## Summary
Addressing critical x86_64 instability where `yield()` causes corruption of the syscall return context.

## Active Plan
Executing `phase-3.md` (Design) and `phase-4.md` (Implementation).

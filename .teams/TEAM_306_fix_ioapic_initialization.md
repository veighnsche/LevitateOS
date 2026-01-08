# TEAM_306: Fix IOAPIC Initialization

## **Objective**
Fix a build error in `los_hal` caused by a missing address argument in `IoApic::new()`.

## **Context**
A recent change removed the address argument from `IoApic::new()` in the `IOAPIC` static initialization, but the function signature still requires it.

## **Progress**
- [x] Create team file
- [x] Initialize todo list
- [x] Inspect `crates/hal/src/x86_64/ioapic.rs`
- [x] Fix `IOAPIC` initialization
- [x] Verify build

# Phase 3 — Step 4: ulib and Process Entry

## Parent
[Phase 3: Implementation](phase-3.md)

## Goal
Provide the userspace runtime environment for x86_64 binaries.

## Tasks
1. **x86_64 Entry Point**:
   - Implement `_start` in `userspace/ulib/src/arch/x86_64.rs`.
   - Setup initial stack alignment.
   - Call `main`.
2. **Linker Script**:
   - Ensure `userspace/linker.ld` is compatible with x86_64 or provide an arch-specific version.

## Exit Criteria
- `ulib` builds for x86_64.
- Binaries have a valid ELF entry point for x86_64.

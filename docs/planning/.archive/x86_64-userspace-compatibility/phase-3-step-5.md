# Phase 3 — Step 5: ulib and Process Entry

## Parent
[Phase 3: Implementation](phase-3.md)

## Goal
Provide the userspace runtime environment for x86_64 binaries.

## Tasks
1. **x86_64 Entry Point**:
   - Implement `_start` in `userspace/ulib/src/arch/x86_64.rs`.
   - Setup initial stack alignment (16-byte).
   - Call `main`.
2. **Linker Script**:
   - Create `userspace/ulib/linker_x86_64.ld` or update `userspace/linker.ld`.
   - Ensure typical x86_64 ELF layout (e.g., 2MB page alignment for sections if desired, or standard 4KB).

## Exit Criteria
- `ulib` builds for x86_64.
- Binaries have a valid ELF entry point for x86_64.

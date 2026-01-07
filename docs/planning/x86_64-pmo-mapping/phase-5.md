# Phase 5: Polish, Docs, and Cleanup - x86_64 PMO Mapping

## Cleanup Tasks
- **Remove Identity Mapping**: Once the higher-half transition is stable, reduce the identity mapping in `boot.S` to just the minimum required for the jump (or remove it if we jump directly to high-half long mode).
- **Consolidate Constants**: Ensure all architecture-specific memory constants (UART, VGA, etc.) are defined relative to `DEVICE_VIRT_BASE` or `PHYS_OFFSET`.

## Documentation
- Update `docs/architecture/memory.md` with the new x86_64 layout.
- Add comments to `boot.S` explaining the PML4[256] purpose.

## Handoff Notes
- This implementation serves as the baseline for userspace process creation, which will use the same PMO mechanism to manage user page tables.

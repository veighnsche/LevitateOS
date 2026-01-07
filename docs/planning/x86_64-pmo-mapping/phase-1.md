# Phase 1: Discovery - x86_64 PMO Mapping

## Feature Summary
Implement a Physical Memory Offset (PMO) mapping for x86_64. This maps the entire physical address space (up to a reasonable limit, e.g., 64GB or more) to a high virtual address range.
- **Problem**: Current x86_64 MMU implementation relies on identity mapping (PA == VA) for early boot, which is limited and fragile as more memory is needed for page tables.
- **Benefit**: Allows the kernel to access any physical frame through a simple offset, enabling robust page table management and simplifying drivers that need to access physical memory.

## Success Criteria
- [ ] Entire physical memory (up to 64GB) is accessible via `PHYS_OFFSET + PA`.
- [ ] MMU walker and mapper in Rust use this offset instead of identity mapping.
- [ ] Identity mapping in `boot.S` can be safely removed or minimized after transition.
- [ ] `virt_to_phys` and `phys_to_virt` are updated to handle the offset.

## Current State Analysis
- **x86_64 Boot**: `boot.S` sets up a 16MB identity mapping and a -2GB higher-half mapping for the kernel binary.
- **MMU Core**: `crates/hal/src/x86_64/mmu.rs` and `paging.rs` assume identity mapping for sub-table access.
- **AArch64 Parity**: AArch64 already uses a similar mechanism (though usually lower-memory based in many kernels, LevitateOS might have its own pattern).

## Codebase Reconnaissance
- `crates/hal/src/x86_64/mmu.rs`: Contains `virt_to_phys`, `phys_to_virt`, and `map_page`.
- `crates/hal/src/x86_64/paging.rs`: Contains the page table walker (`walk_mut`) which is the primary consumer of identity mapping currently.
- `kernel/src/arch/x86_64/boot.S`: Initial page table setup.

## Constraints
- **Address Space**: Must choose a high virtual address that doesn't conflict with the kernel (-2GB) or userspace. `0xFFFF800000000000` is a common choice (canonical high half).
- **Performance**: Must use Huge Pages (1GB or 2MB) for the PMO mapping to minimize TLB pressure.
- **Bootloader**: Depends on the memory map provided by Multiboot2.

## Steps
1. **Step 1 - Analyze AArch64 Implementation**: Check how AArch64 handles physical memory access to maintain architecture parity.
2. **Step 2 - Define Address Space Layout**: Choose the virtual base for PMO.
3. **Step 3 - Draft Mapping Strategy**: Decide between early assembly mapping or late Rust mapping.

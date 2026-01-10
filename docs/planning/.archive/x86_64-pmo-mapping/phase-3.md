# Phase 3: Implementation - x86_64 PMO Mapping

## Implementation Overview
We will implement the PMO mapping in three stages:
1. **HAL Foundation**: Introduce fixed constants and update `virt_to_phys` / `phys_to_virt`.
2. **Boot Assembly**: Set up the PML4 entry and early leaf mappings in `boot.S`.
3. **Rust MMU Refinement**: Update the walker to use `phys_to_virt` and implement expansion logic.

## Steps

### Step 1: HAL Foundation
Update `crates/hal/src/x86_64/mmu.rs` with `PHYS_OFFSET` and updated conversion functions.

### Step 2: Boot Assembly
Modify `kernel/src/arch/x86_64/boot.S`:
- Add `pmo_pdpt` and `pmo_pd` symbols.
- Set `early_pml4[256]` to `pmo_pdpt`.
- Map `pmo_pdpt[0]` to `pmo_pd`.
- Identity map the first 1GB of physical memory to `pmo_pd` using 2MB huge pages.

### Step 3: MMU Refactor
Refactor `crates/hal/src/x86_64/paging.rs`:
- Update `walk` and `walk_mut` to use `phys_to_virt` for sub-table access.
- Remove identity mapping assumptions.

### Step 4: Kernel mapping update
Refactor `crates/hal/src/x86_64/mmu.rs`:
- Update `init_kernel_mappings` to use the linker symbols and map the entire kernel via the higher-half window.

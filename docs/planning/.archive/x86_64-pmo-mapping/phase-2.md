# Phase 2: Design - x86_64 PMO Mapping

## Proposed Solution
Map the entire physical address space to a high virtual address base.
- **PMO Base**: `0xFFFF800000000000` (Architecture Parity with AArch64).
- **Mapping Mechanism**:
  - In `boot.S`, we will introduce a new PML4 entry (e.g., `PML4[256]`) to point to a series of PDPTs that map physical memory.
  - For efficiency during boot, we will map the first 1GB (1 x 1GB huge page if supported, or 512 x 2MB huge pages) to this offset.
  - In Rust `init_kernel_mappings`, we will expand this mapping based on the memory map provided by Multiboot2.

## API Design
Update `crates/hal/src/x86_64/mmu.rs`:

```rust
pub const PHYS_OFFSET: usize = 0xFFFF800000000000;

#[inline]
pub fn phys_to_virt(pa: usize) -> usize {
    pa + PHYS_OFFSET
}

#[inline]
pub fn virt_to_phys(va: usize) -> usize {
    if va >= PHYS_OFFSET {
        va - PHYS_OFFSET
    } else {
        va - KERNEL_VIRT_BASE
    }
}
```

## Behavioral Decisions
- **Unaligned Access**: Page table entries must still be 4KB aligned. PMO mapping will use 1GB or 2MB pages where possible.
- **Overlap**: `0xFFFF800000000000` + 64GB = `0xFFFF801000000000`. This does not overlap with the kernel binary at `0xFFFFFFFF80000000`.
- **Early Boot Consistency**: `boot.S` must ensure the PMO mapping is present before `kernel_main` attempts to use `WalkResult` which will now rely on `phys_to_virt`.

## Open Questions
1. **PML4 Index**: Which PML4 index should we use for `0xFFFF800000000000`?
   - Calculations: `(0xFFFF800000000000 >> 39) & 0x1FF = 256`.
   - `PML4[256]` will be our PMO anchor.
2. **Huge Page Support**: Does all x86_64 hardware support 1GB pages?
   - No, it's an optional feature. We should use 2MB pages (supported by all 64-bit CPUs) for maximum compatibility in early boot, or check `CPUID` and fallback.
   - For early boot simplicity, 2MB pages are easiest since `boot.S` already has logic for them.

## Steps
1. **Step 1 - Define Constants**: Add `PHYS_OFFSET` to `mmu.rs`.
2. **Step 2 - Update Pointer Arithmetic**: Refactor `paging.rs` and `mmu.rs` to use `phys_to_virt`.
3. **Step 3 - Implement Boot Mappings**: Update `boot.S` to set up `PML4[256]`.

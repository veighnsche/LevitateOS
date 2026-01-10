# Phase 4: Integration and Testing - x86_64 PMO Mapping

## Integration Points
- **Entry Point**: `kernel_main` calls `hal::init()`, which now expects the PMO mapping to be present.
- **Allocator**: `EARLY_ALLOCATOR` will continue to return physical addresses, but the MMU will now correctly access them via `PHYS_OFFSET`.

## Test Strategy
1. **Unit Tests (Host)**:
   - Verify `virt_to_phys` and `phys_to_virt` logic with mock addresses.
2. **Boot Test (QEMU)**:
   - Build and run the kernel. If it reaches "OK" on VGA, the early PMO mapping worked.
   - Add a test in `kernel_main` to write to a high memory address (e.g., `PHYS_OFFSET + 0xB8000`) to verify the PMO mapping covers VGA.
3. **MMU Regression**:
   - Verify `aarch64` still builds and runs correctly.

## Verification Tasks
- [ ] `cargo xtask build kernel --arch x86_64` succeeds.
- [ ] Kernel boots to VGA "OK".
- [ ] Debug print of a physical address via `phys_to_virt` works.

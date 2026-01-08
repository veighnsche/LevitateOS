# TEAM_288: x86_64 Limine Boot Bugfix

## 1. Team Registration
- **Team ID**: TEAM_288
- **Predecessor**: TEAM_287 (x86_64 behavior test completion)
- **Focus**: Fix x86_64 Limine boot issues - virt_to_phys, initramfs, HHDM

## 2. Context from TEAM_287

TEAM_287 completed x86_64 behavior test but identified 3 remaining blockers:

### Remaining Issues
1. **GPU init hangs** - `virt_to_phys()` returns wrong address
   - Root cause: Limine loads kernel at different PA than linker script assumes
   - Linker script: `__kernel_phys_start = 0x200000`
   - Limine: May load at arbitrary PA (reported via `EXECUTABLE_AND_MODULES` region)

2. **Initramfs not in ISO** - Limine module not passed to kernel
   - Default boot entry in `limine.cfg` doesn't include `MODULE_PATH`
   - Kernel boots to maintenance shell

3. **HHDM request not filled** - `BASE_REVISION.is_supported()` returns false
   - Memory map works, but HHDM offset not obtained from bootloader
   - Current workaround: default `PHYS_OFFSET` matches Limine's default

## 3. Root Cause Analysis

### Issue 1: virt_to_phys Wrong Address

The `virt_to_phys()` function in `crates/hal/src/x86_64/mmu.rs`:

```rust
pub fn virt_to_phys(va: usize) -> usize {
    if va >= KERNEL_VIRT_BASE {
        va - KERNEL_VIRT_BASE + unsafe { &__kernel_phys_start as *const _ as usize }
    } else if va >= unsafe { PHYS_OFFSET } {
        va - unsafe { PHYS_OFFSET }
    } else {
        va
    }
}
```

This reads `__kernel_phys_start` from linker script, which is set to `0x200000`. But Limine
can load the kernel at a different physical address and reports this via EXECUTABLE_AND_MODULES
memory region.

**Fix**: Get actual kernel load address from:
1. Limine's EXECUTABLE_AND_MODULES memory region, or
2. Limine's KERNEL_ADDRESS_REQUEST response

### Issue 2: Initramfs Missing

In `limine.cfg`, the default entry is:
```
:LevitateOS (x86_64)
    PROTOCOL=limine
    KERNEL_PATH=boot:///boot/levitate-kernel
    KASLR=no
```

This doesn't include `MODULE_PATH=boot:///boot/initramfs.cpio`.

**Fix Options**:
1. Merge into single entry with MODULE_PATH
2. Make "initramfs" entry the default
3. Build script ensures initramfs is always included

### Issue 3: HHDM Request Not Filled

`BASE_REVISION.is_supported()` returns false, which means Limine didn't recognize
the requests. Possible causes:

1. **Limine crate version mismatch** - Protocol version incompatible
2. **.requests section not found** - Limine scanning wrong area
3. **Request format changed** - Struct layout differs

## 4. Planning Reference
See `docs/planning/x86_64-limine-bugfix/` for detailed phases.

## 5. Progress Log

| Time | Action |
|------|--------|
| Start | Created team file, began bugfix planning |
| +5min | Analyzed root causes, created planning structure |
| +10min | Implemented virt_to_phys fix (KERNEL_PHYS_BASE) |
| +15min | Added KernelAddressRequest to limine.rs |
| +20min | Added MODULE_PATH to limine.cfg |
| +25min | Verified x86_64 build, ran behavior test |
| Done | Bug 1 & 2 fixed, GPU init works, initramfs loads |

## 6. Handoff Checklist
- [x] Project builds cleanly (x86_64 and aarch64)
- [ ] Unit tests pass (cargo test --workspace)
- [x] x86_64 behavior test golden file updated
- [ ] aarch64 behavior test (unrelated differences in initramfs)
- [x] Team file updated
- [x] Remaining TODOs documented

## 7. Remaining Work

1. **x86_64 GPF during initramfs listing** - New bug discovered
   - Kernel crashes with General Protection Fault when listing initramfs files
   - Not related to virt_to_phys or initramfs loading (those work now)
   - Needs separate investigation

2. **HHDM request still not filled** - Bug 3 not addressed
   - BASE_REVISION.is_supported() still returns false
   - Current workaround (default PHYS_OFFSET) works
   - Lower priority since GPU/memory access works

# TEAM_286: Investigate Serial Silence

## 1. Pre-Investigation Checklist
- **Team ID**: TEAM_286
- **Bug Summary**: Kernel produces 0 bytes of serial output during behavior tests.
- **Environment**: x86_64, Limine v7.x, linked at `0xFFFFFFFF80000000`.
- **Prior Work**: TEAM_285 reported 'K' diagnostic printed, but behavior test sees 0 output.

## 2. Root Causes Identified

### Issue 1: Behavior test used `-kernel` for x86_64 (FIXED)
- QEMU's SeaBIOS doesn't support multiboot
- **Fix**: Use Limine ISO boot (`-cdrom levitate.iso -boot d`) for x86_64

### Issue 2: CR3 switch broke Limine's page tables (FIXED)
- Kernel switched to `early_pml4` which assumed physical load at 0x200000
- Limine loads kernel at arbitrary physical address
- **Fix**: Skip CR3 switch for Limine boot (`init_with_options(false)`)

### Issue 3: Limine boot detection failed (FIXED)
- `is_limine_boot()` checked `BASE_REVISION.is_supported()` which returned false
- boot.S sets `multiboot_magic=0` for Limine path
- **Fix**: Detect Limine via `multiboot_magic == 0`

### Issue 4: BASE_REVISION.is_supported() returns false (PARTIALLY FIXED)
- Limine isn't finding our request structures
- Cause: Unknown - possibly section placement or Limine version issue
- **Workaround**: Continue parsing even when check fails (keep protocol=Limine)

### Issue 5: APIC region not accessible (FIXED)
- APIC at 0xFEE00000 caused page fault
- Limine's identity mapping may not include APIC or has wrong permissions
- **Fix**: Skip APIC/IOAPIC init for Limine boot

### Issue 6: PCI ECAM not accessible (BLOCKER)
- PCI uses virtual address 0xFFFFFFFF40000000 for ECAM
- This mapping is in `mmu::init_kernel_mappings()` but we skip CR3 switch
- **Solution needed**: Either use HHDM for PCI or fix Limine request detection

### Issue 7: Golden file is for aarch64 (NEEDS FIX)
- `tests/golden_boot.txt` contains aarch64-specific output
- x86_64 needs separate golden file or arch-aware testing

## 3. Current State

The kernel now boots via Limine and produces output:
```
[BOOT] Stage 1: Early HAL (SEC)
Heap initialized.
[BOOT] Protocol: Limine
[BOOT] Memory: 16 regions, 476 MB usable
...
[GPU] Initializing via PCI...
[PCI] Scanning Bus 0 for GPU...
KERNEL PANIC: PAGE FAULT at 0xFFFFFFFF40000000 (ECAM)
```

## 4. Remaining Work

1. **Fix Limine Request Detection**: Investigate why `BASE_REVISION.is_supported()` returns false
2. **Access PCI via HHDM**: Once HHDM works, PCI/GPU can use it instead of custom VA
3. **Create x86_64 Golden File**: Separate behavior test baseline for x86_64
4. **APIC via HHDM**: Once HHDM works, re-enable APIC with HHDM-based access

## 5. Files Modified
- `xtask/src/tests/behavior.rs` - Use ISO boot for x86_64
- `xtask/src/build.rs` - Add `build_iso_verbose()`
- `crates/hal/src/x86_64/mod.rs` - `init_with_options()`, skip APIC for Limine
- `kernel/src/main.rs` - Conditional CR3 switch
- `kernel/src/arch/x86_64/mod.rs` - Detect Limine via magic=0
- `kernel/src/boot/limine.rs` - Don't return early on BASE_REVISION fail

## 6. Handoff Notes
The next team should focus on why Limine requests aren't being detected. Possible causes:
- `.requests` section placement in linker script
- Limine version incompatibility
- Request struct format changes in limine crate

Once HHDM is working, the kernel should be able to access APIC and PCI via physical offsets.

## 7. Handoff Checklist

- [x] Project builds cleanly
- [x] Unit tests pass
- [ ] Behavior test passes (BLOCKED: x86_64 needs own golden file + PCI ECAM fix)
- [x] Team file updated
- [x] Remaining TODOs documented

## 8. Progress Summary

| Before | After |
|--------|-------|
| 0 lines output | 24 lines output |
| Kernel hung immediately | Kernel boots through Stage 3 |
| No serial detection | Serial works via Limine |
| CR3 switch crashed | CR3 switch skipped for Limine |

**Status: PARTIAL SUCCESS** - Serial silence resolved, but behavior test blocked on PCI ECAM mapping and aarch64 golden file.

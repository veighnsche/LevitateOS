# TEAM_277: x86_64 Kernel Syscall Handler

## Goal
Implement x86_64 kernel syscall handler to enable userspace tests.

## Status: PARTIAL - BOOTS BUT PANICS ⚠️

## Completed
- ✅ `syscall.rs` - MSR init (IA32_STAR, IA32_LSTAR, IA32_FMASK, EFER.SCE)
- ✅ `syscall.rs` - Naked `syscall_entry` using `naked_asm!`
- ✅ `syscall.rs` - Handler calls `syscall_dispatch()`
- ✅ `mod.rs` - `SyscallFrame` with x86_64 register layout
- ✅ `task.rs` - `cpu_switch_to` via global_asm
- ✅ `task.rs` - `task_entry_trampoline` stub
- ✅ **Fixed Boot Issue**: `boot.S` now accepts Multiboot1 (QEMU) & Multiboot2 (GRUB).

## Current Blocker: APIC Page Fault
The kernel boots and produces serial output (detected via `run-term.sh`), but **panics** during initialization:
```
KERNEL PANIC: panicked at crates/hal/src/x86_64/exceptions.rs:116:5:
EXCEPTION: PAGE FAULT
Accessed Address: fee000f0 (APIC Base + Register)
```
**Root Cause**: The Local APIC memory region (`0xFEE00000`) is not mapped in the page tables.

## Next Steps for Team
1. **Fix APIC Mapping**: Add mapping for `0xFEE00000` (PAGE_SIZE) in `kernel/src/arch/x86_64/mmu.rs` -> `init_kernel_mappings_refined` (or `early_init`).
2. **Verify Boot**: Retest with `cargo xtask run test --arch x86_64`.
3. **Userspace Init**: Once kernel fully inits, implementing `enter_user_mode` and user GDT segments to run `init` process.

## Handoff
- [x] x86_64 kernel compiles and boots
- [x] Syscall infrastructure ready (MSRs, Handler)
- [x] Bootloader compatibility fixed (Multiboot1/2)
- [ ] **IMMEDIATE FIX NEEDED**: Map APIC MMIO region to stop panic.

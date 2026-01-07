# Phase 1: Discovery and Safeguards

## Refactor Summary

**What:** Replace the current "patch on patch" x86_64 boot assembly with a clean, modular boot abstraction that works across architectures and boot protocols.

**Why (Pain Points):**
1. `boot.S` is 330 lines of complex assembly doing bootloader work
2. Manual 32→64 bit transition we shouldn't need with modern bootloaders
3. Recent fixes (APIC mapping, register preservation) are patches on a flawed design
4. No path to real UEFI hardware (Intel NUC)
5. x86_64 and AArch64 have completely different boot paths with no abstraction

**UNIX Philosophy Violations in Current Code:**
- **Rule 1 (Modularity)**: boot.S does page tables, GDT, mode transition, BSS clearing - too many responsibilities
- **Rule 2 (Composition)**: Multiboot info struct is x86-specific, not composable
- **Rule 5 (Safety)**: 330 lines of unsafe assembly with no safe wrapper

---

## Success Criteria

| Criterion | Before | After |
|-----------|--------|-------|
| **Boot abstraction** | None - arch-specific paths | Unified `BootInfo` struct |
| **Assembly lines** | 330 lines in boot.S | ~20 lines (entry stub only) |
| **Real hardware** | QEMU only | NUC boots via UEFI |
| **Architectures** | Separate x86_64/AArch64 paths | Same `kernel_main(&BootInfo)` |
| **Bootloader coupling** | Hardcoded Multiboot1/2 | Pluggable (Limine, DTB, etc.) |

---

## Behavioral Contracts (What Must Not Break)

### Current Boot Contracts
1. **x86_64 QEMU**: `qemu -kernel` → kernel boots → serial output
2. **AArch64 QEMU**: `qemu -kernel` → kernel boots → serial output  
3. **Kernel entry**: `kernel_main(magic, info_ptr)` receives boot info
4. **Memory detection**: Kernel can discover available RAM
5. **Higher-half**: Kernel runs at `0xFFFFFFFF80000000` (x86_64)

### Test Commands
```bash
# x86_64 boot test
timeout 5 qemu-system-x86_64 -M q35 -cpu qemu64 -m 1G \
  -kernel target/x86_64-unknown-none/release/levitate-kernel \
  -nographic -serial mon:stdio -no-reboot

# AArch64 boot test  
timeout 5 qemu-system-aarch64 -M virt -cpu cortex-a72 -m 1G \
  -kernel kernel64_rust.bin -nographic -serial mon:stdio -no-reboot

# Full behavior test
cargo xtask test behavior
```

---

## Golden/Regression Tests

### Baseline Outputs to Preserve
1. **x86_64 boot log**:
   ```
   [BOOT] x86_64 kernel starting...
   [BOOT] Booted via Multiboot1 (QEMU)
   [MEM] Buddy Allocator initialized
   [BOOT] x86_64 kernel initialized
   ```

2. **AArch64 boot log**: (check `tests/golden_boot.txt`)

3. **Behavior tests**: `cargo xtask test behavior` must pass

### Lock-in Actions
- [ ] Capture current x86_64 boot output as golden log
- [ ] Ensure AArch64 golden logs are current
- [ ] Run full test suite before any changes

---

## Current Architecture Analysis

### x86_64 Boot Flow (Current)
```
QEMU -kernel
    │
    ▼
boot.S (_start, 32-bit protected mode)
    │
    ├── Save multiboot magic/info to stack (TEAM_278 fix)
    ├── Setup GDT64
    ├── Call setup_early_page_tables (identity + higher-half + PMO + APIC)
    ├── Enable PAE, set CR3, enable long mode
    ├── Far jump to long_mode_start
    │
    ▼
boot.S (long_mode_start, 64-bit)
    │
    ├── Reset segment registers
    ├── Set RSP to higher-half stack
    ├── Save multiboot args to R12/R13 (TEAM_278 fix)
    ├── Zero BSS
    ├── Restore multiboot args
    ├── Call kernel_main(magic, info_ptr)
    │
    ▼
kernel_main (mod.rs)
    │
    ├── Write "OK" to VGA
    ├── Init heap
    ├── Init HAL (serial, VGA, IDT, APIC, PIT)
    ├── Parse multiboot info (if multiboot2)
    ├── Init memory (buddy allocator)
    └── Halt loop
```

### AArch64 Boot Flow (Current)
```
QEMU -kernel
    │
    ▼
_start (boot.rs, already 64-bit EL1)
    │
    ├── Set SP
    ├── Zero BSS
    ├── Call rust_main(dtb_ptr)
    │
    ▼
rust_main (main.rs)
    │
    ├── Parse DTB for memory info
    ├── Init HAL
    ├── Init scheduler
    └── Run init process
```

### Key Observation
**AArch64 is already clean** - the bootloader (QEMU) does the hard work. x86_64 does way too much in boot.S because Multiboot gives us 32-bit protected mode.

---

## Dependency Graph

```
                    ┌─────────────────┐
                    │   Bootloader    │
                    │ (QEMU/Limine)   │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
    ┌─────────▼─────────┐       ┌──────────▼──────────┐
    │  boot.S (x86_64)  │       │  boot.rs (AArch64)  │
    │  330 lines asm    │       │  ~50 lines Rust     │
    └─────────┬─────────┘       └──────────┬──────────┘
              │                             │
              │  Different signatures!      │
              │  (magic, info_ptr)          │  (dtb_ptr)
              │                             │
    ┌─────────▼─────────┐       ┌──────────▼──────────┐
    │ kernel_main()     │       │ rust_main()         │
    │ (x86_64/mod.rs)   │       │ (main.rs)           │
    └─────────┬─────────┘       └──────────┬──────────┘
              │                             │
              └──────────────┬──────────────┘
                             │
                    ┌────────▼────────┐
                    │  Kernel Core    │
                    │ (scheduler,VFS) │
                    └─────────────────┘
```

**Problem**: Two completely different entry paths with no common abstraction.

---

## Constraints

1. **QEMU must keep working** - Development relies on it
2. **AArch64 must not regress** - It's already clean
3. **Incremental migration** - Can't break boot for weeks
4. **Real hardware goal** - Must work on Intel NUC (UEFI)
5. **UNIX philosophy** - Modular, composable, simple

---

## Open Questions

1. **Keep Multiboot path?** - Useful for QEMU iteration, or fully migrate to Limine?
2. **Limine for AArch64?** - Use Limine for both, or keep DTB path for AArch64?
3. **Timeline** - NUC boot priority vs other kernel features?

---

## Steps

### Step 1: Lock in Current Behavior
- Run all tests, capture baselines
- Document exact boot output for both architectures
- Ensure behavior tests pass

### Step 2: Audit Boot Code Inventory
- List all files involved in boot
- Count lines of unsafe assembly
- Identify what bootloader should do vs kernel

### Step 3: Define BootInfo Contract
- Design the unified `BootInfo` struct
- Map Multiboot → BootInfo
- Map DTB → BootInfo
- Map Limine → BootInfo

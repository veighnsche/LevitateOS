# TEAM_285: Investigate LH Hang

## 1. Pre-Investigation Checklist
- **Team ID**: TEAM_285
- **Bug Summary**: Kernel hangs after serial output "LH" during Limine boot migration.
- **Environment**: x86_64, Limine v7.x, linked at `0xFFFFFFFF80000000`.

## 2. Phase 1 — Understand the Symptom
- **Expected Behavior**: Kernel should continue initialization and reach `kernel_main_unified`.
- **Actual Behavior**: Kernel prints "LH" via serial but hangs immediately after.
- **Delta**: Hang occurs between "LH" output in `boot.S` and reaching the unified main.

## 3. Phase 2 — Form Hypotheses
1. **Hypothesis 1**: Serial port (COM1 at `0x3f8`) is not mapped or becomes inaccessible after the kernel switches to its own PML4.
2. **Hypothesis 2**: The jump to the higher-half address of `kernel_main_unified` is failing due to incorrect mapping or relocation.
3. **Hypothesis 3**: An exception is occurring (e.g., Page Fault) but the IDT is not yet set up, leading to a triple fault or silent hang in QEMU.

## 4. Phase 3 — Test Hypotheses with Evidence (Continued)
- **Hypothesis 1 (Serial Port mapping)**: Confirmed. The serial port works before the switch but we need to ensure it's mapped correctly in the new tables.
- **Hypothesis 2 (Higher-half jump)**: Ruled out. The jump to `kernel_main` works perfectly.
- **Hypothesis 3 (Exceptions)**: Confirmed. A page fault was occurring due to missing APIC mappings in `mmu.rs` and incorrect mappings in `boot.S`.

## 5. Phase 4 — Narrow Down to Root Cause
### 5.1. Root Cause Analysis
1.  **Inconsistent Multiboot1 Addresses**: `boot.S` used `0x100000` while `linker.ld` defined `_phys_offset` as `0x200000`. This caused QEMU to reject the header.
2.  **Limine HHDM Offset**: `PHYS_OFFSET` was hardcoded to `0xFFFF800000000000`, but Limine can provide any offset. This caused `virt_to_phys` to fail.
3.  **Assembly CR3 Reload**: Immediate `mov cr3` in assembly was too risky/early. Moving it to Rust allowed for verified initialization.
4.  **APIC Mapping Bug**: `boot.S` mapped Local APIC virtual address to IOAPIC physical address.
5.  **Read-Only Requests**: Limine requests were in `.boot_text` (RX), but Limine needs to write responses to them.

### 5.2. Causal Chain
- Incorrect APIC mapping in `boot.S` -> Kernel starts initialization -> Accesses Local APIC -> Page Fault -> Hang (silent without IDT).
- Hardcoded `PHYS_OFFSET` -> `virt_to_phys` returned wrong physical address for `early_pml4` -> `mov cr3` loaded garbage -> Crash/Hang.

## 6. Phase 5 — Decision: Handoff
The system is now reaching `kernel_main_unified` and initializing the HAL. Diagnostic characters `XLP RK` are printed to serial.
Current issue: The behavior test reports 0 output, meaning the serial port might be silent or incorrectly configured after the page table switch.

## 7. Remaining TODOs
- [ ] **Serial Silence**: Fix COM1 accessibility after switching to kernel page tables.
- [ ] **Memory Manager**: Finalize parsing of Limine memory map into `BootInfo`.
- [ ] **Golden Logs**: Refresh `tests/golden_boot.txt` once output is restored.
- [ ] **Library Migration**: Replace manual arch logic with standard crates.

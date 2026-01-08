# TEAM_289: Investigate x86_64 GPF During Initramfs Listing

## 1. Team Registration
- **Team ID**: TEAM_289
- **Predecessor**: TEAM_288 (x86_64 Limine bugfix)
- **Focus**: Investigate General Protection Fault during initramfs file listing

## 2. Bug Report

### Symptom
x86_64 kernel crashes with General Protection Fault when listing files in initramfs.

### Error Details
```
KERNEL PANIC: panicked at crates/hal/src/x86_64/exceptions.rs:108:5:
EXCEPTION: GENERAL PROTECTION FAULT
Error Code: 0
ExceptionStackFrame {
    instruction_pointer: 18446744071562371484,
    code_segment: 8,
    cpu_flags: 65667,
    stack_pointer: 18446744071562445136,
    stack_segment: 16,
}
```

### Last Successful Output
```
Initramfs found at 0xffff80001dd19000 - 0xffff80001dd5f600 (288256 bytes)
Files in initramfs:
```

### Environment
- Architecture: x86_64
- Boot: Limine v7.x
- Protocol: Limine (not Multiboot2)

## 3. Pre-Investigation Context

### What Works
- Limine boots kernel successfully
- Memory/MMU initialization works
- GPU initialization via PCI works
- Terminal initialized
- Initramfs detected and address range identified

### What Fails
- Crash occurs immediately after "Files in initramfs:" message
- Suggests crash in initramfs file listing/iteration code

## 4. Hypotheses

| # | Hypothesis | Confidence | Evidence Needed |
|---|------------|------------|-----------------|
| H1 | Invalid memory access in CPIO parsing | Medium | Check initramfs VA translation |
| H2 | Stack overflow during iteration | Low | Check stack depth |
| H3 | Misaligned access in x86_64 | Medium | Check struct packing |
| H4 | Bad pointer in iterator | Medium | Trace first iteration |

## 5. Investigation Log

| Time | Action | Finding |
|------|--------|---------|
| Start | Created team file | - |
| +5min | Analyzed IP/SP | Both valid kernel addresses |
| +10min | Traced to init.rs:387 | Crash in archive.iter() loop |
| +15min | Analyzed CpioIterator | pointer arithmetic, but struct is byte-aligned |
| +20min | Checked golden file | initramfs at 0xffff8000... (VA!) |
| +25min | **ROOT CAUSE** | module.addr() returns VA, phys_to_virt double-translates |
| +30min | Applied fix | Check if addr >= 0xFFFF_0000... before translating |
| +35min | Verified | GPF fixed! 70 lines output, files listed |

## 6. Root Cause

**Double VA Translation:** Limine's `module.addr()` returns a virtual address (HHDM-mapped), but `init_userspace()` called `phys_to_virt()` on it, causing invalid memory access.

## 7. Fix

**File:** `kernel/src/init.rs:375-383`
```rust
let initrd_va = if start >= 0xFFFF_0000_0000_0000 {
    start  // Already virtual
} else {
    los_hal::mmu::phys_to_virt(start)
};
```

## 8. Handoff Checklist
- [x] Team file updated
- [x] Root cause identified
- [x] Fix applied and tested
- [x] Golden file updated (70 lines)
- [x] No breadcrumbs left (fix is complete)


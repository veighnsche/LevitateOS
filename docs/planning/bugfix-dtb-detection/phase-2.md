# Bugfix: DTB Detection Failure - Phase 2

## Root Cause Analysis

### Primary Root Cause: CONFIRMED

**QEMU does not pass DTB address in x0 for ELF kernels that don't look like Linux kernels.**

QEMU checks the kernel image header to determine boot protocol:
- **Linux kernel boot protocol** (raw binary): DTB address passed in x0
- **Generic ELF boot** (our case): DTB placed at start of RAM (0x4000_0000), x0 not set

### Root Cause Location

**File:** `kernel/src/main.rs`  
**Lines:** 44-55 (the `_head` kernel header)

```asm
_head:
    b       _start
    .long   0            // code1
    .quad   0x0          // text_offset = 0 ← PROBLEM
    .quad   _end - _head // image_size
    .quad   0x0A         // flags
    .quad   0            // res2
    .quad   0            // res3
    .quad   0            // res4
    .ascii  "ARM\\x64"   // magic
    .long   0            // res5
```

### Why `text_offset=0` is Wrong

1. **ARM64 boot protocol** specifies `text_offset` as the offset from a 2MB-aligned RAM base where the kernel should be loaded.
2. When `text_offset=0` AND `image_size` is set, bootloaders may interpret this inconsistently.
3. QEMU may not recognize this as a valid Linux ARM64 image header.

### Secondary Issue: DTB at 0x4000_0000 Reads as Zero

Even with the fallback check, `0x4000_0000` contains `0x0` instead of DTB magic.
This suggests QEMU places the DTB **after** the kernel load region for ELF boots,
not at the fixed start-of-RAM location.

### Hypothesis Validation

| Hypothesis | Status | Evidence |
|------------|--------|----------|
| x0 not passed for ELF | **CONFIRMED** | `BOOT_REGS` shows all zeros |
| text_offset=0 causes issue | **SUSPECTED** | Needs testing with fix |
| DTB placed after kernel | **SUSPECTED** | 0x4000_0000 is zero |

---

## Investigation Details

See `.teams/TEAM_036_investigate_initramfs_crash.md` for full investigation log.

### Key Evidence
1. QEMU `dumpdtb=` generates valid DTB (verified)
2. Kernel ELF has ARM64 magic header (verified via objdump)
3. Assembly correctly saves x0 → BOOT_DTB_ADDR (verified via objdump)
4. x0=0 at runtime (reproduced)

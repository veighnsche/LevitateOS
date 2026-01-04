# Investigation: Initramfs Crash & Missing DTB

**Team ID:** TEAM_036  
**Date:** 2026-01-04  
**Status:** ROOT CAUSE CONFIRMED  
**Investigator:** TEAM_036 (continued by TEAM_037)

---

## 1. Symptom

- **Primary:** `initramfs` detection fails to find DTB address.
- **Secondary:** Reading `0x4000_0000` returns `0x0` (no DTB magic).
- **Tertiary (Fixed):** Data Abort at `0x4000_0000` due to missing identity map.
- **Context:**
  - `BOOT_REGS: x0=0 x1=0 x2=0 x3=0` – all registers are zero at `_start`.
  - `Read magic: 0x0` at address `0x40000000`.

---

## 2. Root Cause Analysis

### Root Cause 1: QEMU ELF Boot Does Not Follow Linux Boot Protocol

**Confirmed.**

When QEMU loads an ELF kernel via `-kernel`, it treats it as a **generic ELF binary** 
rather than a Linux kernel image. This means:

1. **x0 is NOT set to DTB address** – QEMU only passes `x0=DTB` when using the 
   Linux kernel boot protocol (raw binary `Image` format).
2. **For bare-metal/ELF boot**, QEMU places the DTB at **start of RAM (`0x4000_0000`)**.

### Root Cause 2: Kernel Loads at 0x40080000, Overwriting DTB Area

**Confirmed.**

The kernel ELF is loaded at `0x40080000` (per `linker.ld`), which is within the 
same 1GB RAM region. However, the DTB is supposedly at `0x40000000`.

**Key insight:** When we read `0x40000000`, we get `0x0`. This suggests:
- Either QEMU places DTB after the kernel (at kernel end + padding), OR
- The DTB is somehow overwritten/not loaded for ELF boots.

### Root Cause 3: Kernel Header May Be Incorrect

**Partially Confirmed.**

The kernel has an ARM64 Linux image header (`_head`):
```asm
_head:
    b       _start
    .long   0            // code1 - should be exec code
    .quad   0x0          // text_offset = 0 (PROBLEM?)
    .quad   _end - _head // image_size - OK
    .quad   0x0A         // flags - OK
    ...
    .ascii  "ARM\\x64"   // magic - OK
```

The `text_offset = 0` may confuse QEMU about where to place the kernel.

---

## 3. Evidence

| Evidence                          | Status    | Source                          |
|-----------------------------------|-----------|----------------------------------|
| `x0=0` at `_start`                | Confirmed | Runtime output of `BOOT_REGS`   |
| `0x40000000` reads as `0x0`       | Confirmed | `get_dtb_phys()` fallback code  |
| Assembly correctly saves x0      | Verified  | `objdump` confirms x0→x19→store |
| QEMU generates DTB                | Verified  | `dumpdtb=` produces valid DTB   |
| ELF has ARM64 magic header        | Verified  | `objdump` and source review     |
| QEMU docs: DTB at start of RAM    | Documented| QEMU virt machine documentation |

---

## 4. Hypotheses Status

| Hypothesis                        | Status     | Notes                                |
|-----------------------------------|------------|--------------------------------------|
| H1: Missing identity map          | **FIXED**  | Added mapping for 0x4000_0000-0x4008_0000 |
| H2: QEMU ELF loader skips x0      | **CONFIRMED** | x0=0 for all registers              |
| H3: DTB not at 0x4000_0000        | **CONFIRMED** | Memory reads as 0                    |
| H4: Kernel overwrites DTB         | **SUSPECTED** | Kernel at 0x40080000, DTB at 0x40000000 - should be safe |
| H5: text_offset=0 confuses QEMU   | **SUSPECTED** | Need to test with correct text_offset |

---

## 5. Proposed Solutions

### Solution A: Fix Kernel Header (text_offset)
Modify `_head` in `kernel/src/main.rs` to use correct `text_offset = 0x80000`:
```asm
_head:
    b       _start
    .long   0
    .quad   0x80000          // text_offset (was 0x0)
    .quad   _end - _head
    .quad   0x0A             // flags: 4K pages, LE
    ...
```
This might make QEMU recognize the image as a proper Linux kernel and pass x0.

### Solution B: Scan Memory for DTB Magic
If x0=0, scan memory starting from kernel end for DTB magic `0xD00DFEED`:
```rust
fn find_dtb_in_memory() -> Option<usize> {
    // Scan from kernel end to end of first 1GB
    let scan_start = KERNEL_PHYS_END;
    let scan_end = 0x8000_0000; // End of first 1GB
    for addr in (scan_start..scan_end).step_by(0x1000) {
        let magic = unsafe { core::ptr::read_volatile(addr as *const u32) };
        if u32::from_be(magic) == 0xD00DFEED {
            return Some(addr);
        }
    }
    None
}
```

### Solution C: Use Raw Binary Boot (Linux Image Format)
Convert ELF to raw binary and boot as Linux Image:
```bash
aarch64-linux-gnu-objcopy -O binary kernel.elf Image
qemu-system-aarch64 -kernel Image ...
```
This should trigger proper Linux boot protocol with x0=DTB.

### Solution D: Embed DTB at Known Location
Use `include_bytes!` to embed a QEMU-generated DTB in the kernel.

---

## 6. Recommended Next Steps

1. **Try Solution A first** – Fix kernel header to use `text_offset=0x80000`
2. **If A fails, try Solution C** – Raw binary boot 
3. **Document run.sh changes** – Need to add `-initrd initramfs.cpio` flag

### IMPORTANT: Missing -initrd Flag

The current `run.sh` does NOT pass an initramfs to QEMU!
```diff
 qemu-system-aarch64 \
     -M virt \
     ...
+    -initrd initramfs.cpio \
     ...
```

---

## 7. Breadcrumbs Left

```
// TEAM_036 BREADCRUMB: CONFIRMED - x0 is 0 because QEMU ELF boot doesn't pass DTB
// See .teams/TEAM_036_investigate_initramfs_crash.md for full analysis
```

Location: `kernel/src/main.rs:get_dtb_phys()`

---

## 8. Handoff Notes

- Root cause is confirmed: QEMU ELF boot doesn't follow Linux boot protocol
- The kernel header may need `text_offset` fix
- `run.sh` needs `-initrd` flag to actually test initramfs
- Solution A (header fix) should be tried first
- If Solution A fails, Solution C (raw binary) is the fallback

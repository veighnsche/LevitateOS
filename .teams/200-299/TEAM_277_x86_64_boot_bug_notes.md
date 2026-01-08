# TEAM_277: x86_64 QEMU Boot Bug Investigation

## Problem Statement
When running `run-term.sh` for x86_64, the kernel loads but produces no serial output and appears to hang immediately.

---

## Root Cause Found
**boot.S only accepted multiboot2 magic, not multiboot1.**

```asm
/* Check Multiboot2 magic */
cmp eax, 0x36D76289
jne .hang          # <-- HANGS if multiboot1!
```

When booted via QEMU's `-kernel` option with multiboot1:
- EAX contains `0x2BADB002` (multiboot1 magic)
- boot.S expected `0x36D76289` (multiboot2 magic)
- Kernel immediately jumped to `.hang` loop

---

## Investigation Steps Taken

### 1. Verified Kernel Loading
- QEMU accepted the kernel without error messages
- No "Cannot load" or "Error loading" messages

### 2. Checked Multiboot Headers
```bash
objdump -s -j .multiboot1 target/x86_64-unknown-none/release/levitate-kernel
```
- Multiboot1 header at file offset 0x1000 (within first 8KB ✓)
- Magic: 0x1BADB002 ✓
- Checksum: 0xE4514FFB ✓ (-(magic + flags))
- AOUT_KLUDGE flag set with load addresses

### 3. Verified Entry Point
```bash
readelf -h target/x86_64-unknown-none/release/levitate-kernel | grep Entry
# Entry point address: 0x100050
```
This matches `_start` symbol location (after multiboot headers).

### 4. Checked ELF Program Headers
First LOAD segment: file 0x1000 → address 0x100000 (multiboot headers)
Second LOAD segment: file 0x1050 → address 0x100050 (_start)

### 5. Traced Execution
kernel_main writes "OK" to VGA buffer at 0xB8000, but this never appeared.
→ Crash must be in boot.S BEFORE kernel_main is called.

### 6. Found the Bug
boot.S line 85-86: Only multiboot2 magic accepted, multiboot1 rejected.

---

## The Fix
Modify boot.S to accept both magic values:

```asm
/* Check for multiboot2 magic */
cmp eax, 0x36D76289
je .magic_ok

/* Check for multiboot1 magic (QEMU -kernel) */
cmp eax, 0x2BADB002
je .magic_ok

/* Unknown magic - hang */
jmp .hang

.magic_ok:
/* Continue boot... */
```

Also update kernel_main to detect multiboot1 vs multiboot2:
```rust
const MULTIBOOT1_MAGIC: usize = 0x2BADB002;
if multiboot_magic == MULTIBOOT1_MAGIC {
    // Handle multiboot1 boot info structure
}
```

---

## Files Modified
- `kernel/src/arch/x86_64/boot.S` - Accept both multiboot1/2 magic
- `kernel/src/arch/x86_64/linker.ld` - Added .multiboot1 section
- `kernel/src/arch/x86_64/mod.rs` - Added multiboot1 magic detection
- `run-term.sh` - Fixed kernel path to ELF binary

---

## x86_64 Multiboot Summary

| Bootloader | Magic in EAX | Info Struct Format |
|------------|--------------|-------------------|
| QEMU -kernel | 0x2BADB002 | multiboot_info (v1) |
| GRUB2       | 0x36D76289 | multiboot2 boot info |

---

## Remaining Work
1. Parse multiboot1 info structure for memory map
2. Verify kernel continues past HAL init
3. Get serial output working
4. Enable scheduler/init process spawning

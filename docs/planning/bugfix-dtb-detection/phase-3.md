# Bugfix: DTB Detection Failure - Phase 3

## Fix Design and Validation Plan

### Root Cause Summary
The kernel image header (`_head` in `kernel/src/main.rs`) has `text_offset=0`, which
prevents QEMU from recognizing the kernel as a Linux ARM64 image. As a result:
1. QEMU uses "generic ELF boot" instead of Linux boot protocol
2. DTB address is NOT passed in x0 register
3. DTB is placed elsewhere in memory (not at 0x4000_0000)

### Fix Strategy

**Approach:** Correct the ARM64 Linux kernel image header to conform to the boot protocol.

#### Changes Required

1. **Set `text_offset = 0x80000`** (512KB, standard Linux kernel offset)
2. **Verify `flags` field** (currently 0x0A = LE, 4K pages, which is correct)
3. **Add memory scan fallback** as backup (optional, for robustness)
4. **Update `run.sh`** to include `-initrd` flag for testing

#### Why 0x80000?

The ARM64 Linux boot protocol specifies that `text_offset` represents the offset from
a 2MB-aligned RAM base. The default for Linux kernels is `0x80000` (512KB). This leaves
space at the start of RAM for the DTB and other boot data.

Our kernel loads at `0x40080000` = RAM base `0x40000000` + offset `0x80000`, which
is exactly correct for `text_offset=0x80000`.

### Reversal Strategy

If the fix doesn't work:

1. **Revert kernel header change:**
   ```bash
   git checkout kernel/src/main.rs -- # lines 44-55
   ```
2. **Signals to revert:**
   - Kernel fails to boot at all
   - New crashes during boot before UART init
   - x0 still 0 (fix didn't help - need to investigate further)

3. **Alternative if revert needed:**
   - Fall back to Solution B (memory scan) or Solution C (raw binary)

### Test Strategy

#### Verification 1: QEMU Boot Test (Automated Check)

Run the kernel in QEMU and verify x0 is non-zero:

```bash
timeout 3 qemu-system-aarch64 \
    -M virt -cpu cortex-a53 -m 512M \
    -kernel target/aarch64-unknown-none/release/levitate-kernel \
    -display none -nographic -no-reboot 2>&1 | grep -E "BOOT_REGS|magic"
```

**Expected output (after fix):**
```
BOOT_REGS: x0=<non-zero address> x1=0 x2=0 x3=0
Checking magic at <address>
Read magic: 0xd00dfeed  (or BE: 0xedfe0dd0)
```

**Current output (before fix):**
```
BOOT_REGS: x0=0 x1=0 x2=0 x3=0
Read magic: 0x0
```

#### Verification 2: Initramfs Detection Test

1. Create initramfs: `./scripts/make_initramfs.sh`
2. Update `run.sh` to include `-initrd initramfs.cpio`
3. Run kernel and verify initramfs files are listed

**Expected output:**
```
Initramfs found at 0x... - 0x... (... bytes)
Files in initramfs:
 - hello.txt
   Content: Hello from initramfs!
 - test.txt
```

#### Verification 3: Build Verification

```bash
cargo build --release
aarch64-linux-gnu-objdump -d target/aarch64-unknown-none/release/levitate-kernel | head -50
```

Verify the header at `_head` shows:
- `0x40080000`: branch instruction
- Offset +8: `0x80000` (text_offset in little-endian)

### Impact Analysis

- **API Changes:** None
- **Behavior Changes:** DTB/initramfs detection should now work
- **Downstream Impact:** Enables Phase 4+ features (initramfs, device discovery)
- **Hardware Impact:** Improves compatibility with real ARM64 bootloaders (Pixel 6)

---

## Open Questions

None - fix strategy is clear and low-risk.

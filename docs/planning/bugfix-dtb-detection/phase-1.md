# Bugfix: DTB Detection Failure - Phase 1

## Understanding and Scoping

### Bug Summary
- **Short Description:** Kernel fails to detect Device Tree Blob (DTB) at boot. x0 register is zero and reading 0x40000000 returns zero.
- **Severity:** High - blocks initramfs/DTB functionality, critical for hardware boot
- **Impact:** Kernel cannot parse DTB → cannot find initramfs → cannot discover hardware (on real devices)

### Reproduction Status
- **Reproducible:** Yes
- **Steps:**
  1. Build kernel: `cargo build --release`
  2. Run in QEMU: `./run.sh` (or manual qemu-system-aarch64 command)
  3. Observe output: `BOOT_REGS: x0=0 x1=0 x2=0 x3=0`
  4. Observe output: `Read magic: 0x0` at 0x40000000
- **Expected:** x0 should contain DTB physical address, DTB magic should be readable
- **Actual:** x0=0 and no DTB found

### Context
- **Code Areas Affected:**
  - `kernel/src/main.rs` - Kernel header (`_head`) and `get_dtb_phys()` function
  - `linker.ld` - Kernel entry point and load address
- **Recent Changes:** None - this is a long-standing issue
- **Investigation:** See `.teams/TEAM_036_investigate_initramfs_crash.md`

### Constraints
- **Hardware Compatibility:** Fix must work on:
  - QEMU virt machine (development)
  - Pixel 6 (future target) - uses Android Bootloader (ABL)
  - Any ARM64 bootloader following Linux boot protocol
- **Backwards Compatibility:** None - this is fixing a broken feature
- **Time Sensitivity:** Medium - blocks initramfs feature development

### Open Questions
None - root cause is confirmed via TEAM_036 investigation.

---

## Steps (Completed via TEAM_036)

### Step 1: Consolidate Bug Information ✓
Completed in TEAM_036 investigation file.

### Step 2: Confirm Reproduction ✓
Reproduced via `timeout 3 qemu-system-aarch64 ...` command.
Output confirms `x0=0` and `magic=0x0`.

### Step 3: Identify Suspected Code Areas ✓
Root cause isolated to `_head` kernel image header in `kernel/src/main.rs`.

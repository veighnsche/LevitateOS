# Bugfix: DTB Detection Failure - Phase 4

## Implementation and Tests

### Implementation Overview

| Order | File | Change |
|-------|------|--------|
| 1 | `kernel/src/main.rs` | Fix `_head` kernel header: set `text_offset=0x80000` |
| 2 | `run.sh` | Add `-initrd initramfs.cpio` flag |
| 3 | Verification | Run boot test, verify x0 and initramfs detection |

### Reversal Plan

If the fix causes boot failures:

1. **Immediate revert:**
   ```bash
   git diff kernel/src/main.rs  # see what changed
   git checkout kernel/src/main.rs  # revert
   cargo build --release
   ```

2. **Verify revert worked:**
   - Kernel boots (even with broken DTB detection)
   - No new crashes introduced

3. **If original bug reappears:**
   - This is expected after revert
   - Follow breadcrumbs in `.teams/TEAM_036_investigate_initramfs_crash.md`
   - Try alternative solutions B, C, or D

---

## Step 1: Prepare the Codebase

### Tasks
- [ ] Confirm clean git state
- [ ] Confirm kernel builds successfully
- [ ] Record current behavior (baseline)

### Commands
```bash
cd /home/vince/Projects/LevitateOS
git status  # should be clean or changes committed
cargo build --release  # should succeed
```

---

## Step 2: Implement the Fix

### Change 1: Fix kernel header in `kernel/src/main.rs`

**Location:** Lines 44-55

**Current (broken):**
```asm
_head:
    b       _start
    .long   0
    .quad   0x0              // text_offset = 0 (WRONG)
    .quad   _end - _head
    .quad   0x0A
    .quad   0
    .quad   0
    .quad   0
    .ascii  "ARM\\x64"
    .long   0
```

**Fixed:**
```asm
_head:
    b       _start
    .long   0
    .quad   0x80000          // text_offset = 512KB (CORRECT)
    .quad   _end - _head
    .quad   0x0A             // flags: LE, 4K pages
    .quad   0
    .quad   0
    .quad   0
    .ascii  "ARM\\x64"
    .long   0
```

### Change 2: Update run.sh to include initramfs

**Add after line 36 (before `-serial stdio`):**
```bash
-initrd initramfs.cpio \
```

---

## Step 3: Update or Add Tests

No formal test framework exists. Verification will be done via:
1. QEMU boot output inspection
2. `objdump` verification of binary header

---

## Step 4: Run Tests and Verify

### Test 1: Build Verification
```bash
cargo build --release
aarch64-linux-gnu-objdump -d target/aarch64-unknown-none/release/levitate-kernel | head -40
```

**Expected:** See `0x80000` in the header data section.

### Test 2: QEMU Boot Test
```bash
# First create initramfs
./scripts/make_initramfs.sh

# Then run (simplified, no virtio devices)
timeout 5 qemu-system-aarch64 \
    -M virt -cpu cortex-a53 -m 512M \
    -kernel target/aarch64-unknown-none/release/levitate-kernel \
    -initrd initramfs.cpio \
    -display none -nographic -no-reboot 2>&1 | head -30
```

**Success criteria:**
1. `BOOT_REGS: x0=<non-zero>` (DTB address passed)
2. `Read magic: 0xd00dfeed` or `0xedfe0dd0` (DTB found)
3. `Initramfs found at ...` (initrd parsed)
4. `Files in initramfs: - hello.txt` (files listed)

---

## Step 5: Document Reversal Procedure

### Quick Reversal Steps
```bash
# 1. See what changed
git diff kernel/src/main.rs run.sh

# 2. Revert kernel/src/main.rs line 47
# Change: .quad   0x80000
# Back to: .quad   0x0

# 3. Revert run.sh (remove -initrd line if added)

# 4. Rebuild and verify boot works
cargo build --release
```

### Side Effects of Reversal
- DTB detection will be broken again (expected)
- Initramfs will not be detected (expected)
- No data loss or migrations to undo

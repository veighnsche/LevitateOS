# TEAM_152 Summary: qcow2 Boot Failure Fix

## Problem
The qcow2 image was building without errors but failing to boot. Analysis revealed the qcow2 file was only 196 KiB—essentially empty with just partition tables and no actual data. Users wouldn't discover this until trying to boot the image.

## Root Cause
The qcow2 build process completed successfully even when required input files were missing or incomplete:
- No validation that kernel exists
- No validation that install initramfs exists
- No validation that rootfs is complete
- No automatic verification of output
- Work directory deleted before error detection possible

## Solution Implemented

Commit: `632a87d fix(qcow2): add comprehensive validation to prevent empty disk images`

### Phase 1: Fail-Fast Dependency Validation
```rust
fn verify_build_dependencies(base_dir: &Path, rootfs: &Path) -> Result<()>
```
Checks at build start that:
- ✅ Kernel exists at `output/staging/boot/vmlinuz`
- ✅ Install initramfs exists at `output/initramfs-installed.img` (NOT live initramfs)
- ✅ Rootfs has critical directories: usr/bin, usr/lib, etc/shadow, etc/passwd, etc/fstab, boot
- ✅ Rootfs is at least 500 MB

### Phase 2: Diagnostic Output
Shows partition sizes as they're created:
```
Creating EFI partition image...
  EFI partition size: 1024 MB

Creating root partition image (this may take a while)...
  Root partition size: 261118 MB (sparse file)

Assembling disk image...
  Raw disk size: 262142 MB
```
Users can spot if something went wrong by checking if sizes are reasonable.

### Phase 3: Automatic Verification
```rust
fn verify_qcow2_internal(qcow2_path: &Path) -> Result<()>
```
After qcow2 conversion, immediately verifies:
- File size is at least 100 MB (catches empty images)
- Prints: `[OK] Image size: 635 MB (compressed)`

### Phase 4: Work Directory Preservation
If verification fails, work directory is kept:
```
[!] Build verification failed. Work directory preserved for debugging:
    /path/to/output/qcow2-work
  Inspect partition images:
    ls -lh /path/to/output/qcow2-work
```
Users can inspect efi.img, root.img, disk.raw to diagnose issues.

### Phase 5: Documentation
Updated module-level documentation to clarify:
- Build dependencies and order (kernel → initramfs → rootfs → qcow2)
- Complete 10-step process (0 = prerequisites, 1-9 = build steps)
- When to run full build vs individual builds

## Error Messages

### Missing Kernel
```
Error: Kernel not found. Run 'cargo run -- build kernel' first.
```

### Missing Install Initramfs
```
Error: Install initramfs not found. Run 'cargo run -- build initramfs' first.
(The qcow2 requires initramfs-installed.img, not the live initramfs)
```

### Incomplete Rootfs
```
Error: rootfs-staging is incomplete: usr/lib not found.
Run 'cargo run -- build rootfs' to build a complete rootfs.
```

### Too-Small Rootfs
```
Error: rootfs-staging seems too small (12 MB).
A complete rootfs should be at least 500 MB.
Run 'cargo run -- build rootfs' to rebuild.
```

### Verification Failed
```
Error: qcow2 image is suspiciously small (45 MB).
This usually means the build failed to populate the partitions.
Expected size: 500-2000 MB (compressed).

Check that all dependencies were built first:
- cargo run -- build kernel
- cargo run -- build initramfs
- cargo run -- build rootfs
```

## Before/After Comparison

### Before
```
$ cargo run -- build qcow2
=== Building qcow2 VM Image (sudo-free) ===
...
[produces 196 KiB empty image silently]
=== qcow2 Image Built ===
  Output: levitateos-x86_64.qcow2
  Size: 196 MB (sparse)  [Actually 196 KiB, user can't tell]
```
User boots image → "Boot failed: not a bootable disk" ❌

### After
```
$ cargo run -- build qcow2
=== Building qcow2 VM Image (sudo-free) ===

Checking host tools...

Verifying dependencies...
  Kernel not found. Run 'cargo run -- build kernel' first.

Error: qcow2 build requires kernel, install initramfs, and complete rootfs.
Run 'cargo run -- build' to build all artifacts.
```
User immediately knows what's missing ✅

OR (with all dependencies):
```
Creating EFI partition image...
  EFI partition size: 1024 MB

Creating root partition image (this may take a while)...
  Root partition size: 261118 MB (sparse file)

Assembling disk image...
  Raw disk size: 262142 MB

Converting to qcow2 (with compression)...

Verifying qcow2 image...
  [OK] Image size: 635 MB (compressed)

Cleaning up...

=== qcow2 Image Built ===
  Output: levitateos-x86_64.qcow2
  Size: 635 MB (sparse)  [Actually 635 MB, user can trust this]
```
Image boots successfully ✅

## Code Changes

**Files modified:**
- `leviso/src/artifact/qcow2/mod.rs` - Main build logic
- `leviso/src/artifact/qcow2/helpers.rs` - Added `calculate_dir_size()` helper

**Lines added:** 157
**Lines removed:** 11
**Tests passing:** 14/14

## Testing

✅ Code compiles without errors
✅ All unit tests pass
✅ Error messages verified with missing dependencies
✅ Diagnostic output displays correctly
✅ Work directory preservation works
✅ Automatic verification catches undersized images

## User Actions

For users with the boot failure issue:

**To fix immediately:**
```bash
cd leviso
cargo run -- build  # Full build: kernel → initramfs → rootfs → qcow2
```

**If custom builds:**
```bash
cargo run -- build kernel      # Step 1
cargo run -- build initramfs   # Step 2
cargo run -- build rootfs      # Step 3
cargo run -- build qcow2       # Step 4 (now validates all above)
```

**If build still fails:**
```bash
# Inspection helper shows work directory location
ls -lh leviso/output/qcow2-work/  # Check partition sizes
```

## Key Improvements

| Aspect | Before | After |
|--------|--------|-------|
| Fail-fast validation | ❌ None | ✅ Checks dependencies at build start |
| Error clarity | ❌ Silent failure | ✅ Clear "missing X" messages |
| Build visibility | ❌ No intermediate sizes | ✅ Shows each partition size |
| Output verification | ❌ No checks | ✅ Auto-verifies qcow2 after conversion |
| Debugging capability | ❌ Work dir deleted | ✅ Preserved for inspection |
| Build documentation | ⚠️ Unclear order | ✅ Clear 10-step process |

## Related Issues

This fix prevents the "Boot failed: not a bootable disk" error that occurs when:
1. User runs `cargo run -- build qcow2` without building dependencies first
2. Rootfs build produces incomplete filesystem
3. Kernel or initramfs files missing

The fix is backward-compatible and doesn't change the successful build process.

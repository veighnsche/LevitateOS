# Team 304 - Investigate Build ISO Release Failure

## Bug Report
- **Symptom**: `cargo xtask build iso` fails in GitHub Actions with `Error: No such file or directory (os error 2)`.
- **Context**: Error occurs after kernel build succeeds, during ISO creation step.

## Root Cause Analysis

### Phase 1 — Symptom
Error occurred after "x86_64 kernel build complete" with no context on which file was missing.

### Phase 2 — Hypotheses
1. `limine.cfg` missing ❌ (confirmed present)
2. `levitate-kernel` missing ❌ (confirmed present)
3. `initramfs.cpio` missing ❌ (confirmed present)
4. **`limine-bin/` directory exists but files missing ✅ CONFIRMED**

### Phase 3 — Root Cause
**Bug in `prepare_limine_binaries()` (line 345):**

```rust
// Old (buggy):
if !limine_dir.exists() {  // Only checks if directory exists

// New (fixed):
let all_files_exist = files.iter().all(|f| limine_dir.join(f).exists());
if !all_files_exist {      // Checks if all files exist
```

**Problem:** A previous failed run could create the `limine-bin/` directory but not download the files. Subsequent runs would see the directory exists and skip downloading, then fail when trying to copy the missing files.

## Fix Applied
1. Check file existence, not just directory existence
2. Added `-f` flag to curl (fail on HTTP error)
3. Added proper error context to identify failing copies

## Verification
```
[act] ✅  Success - Main Run unit tests 
[act] ✅  Success - Main Build All
[act] ✅  Success - Main Build ISO [3.87s]
```

## Handoff
- [x] Root cause identified
- [x] Fix implemented in `xtask/src/build.rs`
- [x] Verified via act
- [x] Debug output removed

## Part 2 — AArch64 Release Failure
**Symptom**: `act -j build-aarch64` failed at "Build All" step.
**Root Causes & Fixes**:
1. **Userspace Linker Error** (`cannot find crt1.o`):
   - Cause: `aarch64-linux-gnu-gcc` tries to link C runtime startup files unless `-nostartfiles` is used.
   - Fix: Added `cargo:rustc-link-arg=-nostartfiles` to `build.rs` for `init`, `shell`, `levbox`, `repro_crash`, and created `systest/build.rs`.

2. **Missing Entry Points**:
   - `shell`: `_start` was x86_64 specific. Added `#[cfg(target_arch)]` gating and AArch64 `_start`.
   - `repro_crash`: Had `#![no_main]` but defined `main()` instead of `_start`. Renamed to `_start`.

3. **Kernel Build Error (`dtb` module missing)**:
   - Cause: `.gitignore` had `dtb*` which ignored `kernel/src/boot/dtb.rs`. `act` checkout skipped it.
   - Fix: Changed `.gitignore` to `*.dtb` and added `dtb.rs` to git.

4. **Kernel Linker Error**:
   - Cause: Missing `-nostartfiles` and incorrect reference to non-existent `linker.ld`.
   - Fix: Updated `kernel/build.rs` to add `-nostartfiles` for AArch64.

5. **Act Dependencies**:
   - Cause: `cpio`, `fdisk`, `git` missing in `build-aarch64` job.
   - Fix: Updated `release.yml`.

## Final Verification
```
[act] AArch64:
[act] ✅  Success - Main Run unit tests 
[act] ✅  Success - Main Build All
```

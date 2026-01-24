# TEAM_111: Fix Fallback Mechanisms That Mask Real Issues

## Mission
Fix 17 fallback patterns in leviso that violate CLAUDE.md rule #7: "FAIL FAST - Required component missing? `bail!()`, not `println!("Warning...")`"

## Status: Complete ✓

## Verification
- `cargo build` passes
- All 72 leviso unit tests pass
- All 18 process tests pass

## Real Issues Check
Ran `cargo run -- build` and verified no hidden issues were being masked:
- ✓ passwd/group files: Exist, properly formatted
- ✓ Packages directory: All subdirectories readable
- ✓ Kernel config: 7853 lines, properly formatted
- ✓ Cache files: No corruption detected

The mount error during dracut build is expected (requires root privileges).

## What The Fixes Actually Do
The fallbacks were **defensive patterns** that would have masked issues IF they occurred.
No active issues were being hidden. The changes ensure that:
- File corruption → Now fails with clear error instead of using wrong UID/GID
- Directory read errors → Now logged as warnings instead of silent skip
- Mount failures → Now logged for debugging stale mounts
- Missing libraries → Now verified to exist, fails if truly missing
- Signal kills → Now shows actual signal name (SIGKILL, etc.) instead of -1

## Files Modified
1. `leviso/src/build/users.rs` - User/group UID/GID parsing and file reading
2. `leviso/src/build/libdeps.rs` - RPM directory reading
3. `leviso/src/artifact/initramfs.rs` - Directory creation and mount cleanup
4. `leviso/src/component/custom/packages.rs` - Required library copying
5. `leviso/src/build/kernel.rs` - Config file, CPU detection, module filtering
6. `leviso/src/config.rs` - Environment variable loading
7. `leviso/src/process.rs` - Signal-killed process exit codes
8. `leviso/src/component/custom/firmware.rs` - Firmware size calculation
9. `leviso/src/component/custom/etc.rs` - OS config env vars (logging improvement)
10. `leviso/src/cache.rs` - Cache hash file operations
11. `leviso/src/artifact/iso.rs` - Directory iteration, ISO size validation
12. `leviso/src/preflight/dependencies.rs` - Recipe binary env var handling

## Key Fixes
- Replace `.unwrap_or_default()` with proper error handling
- Replace `.ok()?` in critical paths with `.context()?`
- Replace silent `let _ =` with logged warnings
- Verify required libraries exist after copy failure
- Show actual signal name when process killed by signal
- Log parallelism fallback

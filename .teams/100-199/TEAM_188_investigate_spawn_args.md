# TEAM_188: Investigate SpawnArgs Syscall Edge Cases

## Bug Report
- **Component**: `sys_spawn_args` syscall for spawning processes with arguments
- **Task**: Re-enable syscall and proactively find/fix edge cases
- **Source**: User requested investigation
- **Status**: ✅ COMPLETED - Re-enabled syscall, fixed 3 bugs

## Files Modified
- `kernel/src/syscall/mod.rs` - Re-enabled SpawnArgs dispatch
- `kernel/src/syscall/process.rs` - Fixed overflow and deadlock issues

## Bugs Found & Fixed

### Bug 1: argv_size overflow - FIXED
**Location**: `kernel/src/syscall/process.rs:156-161`
**Problem**: `argc * size_of::<UserArgvEntry>()` could overflow
**Fix**: Use `checked_mul()` and return EINVAL on overflow

### Bug 2: entry_ptr calculation overflow - FIXED
**Location**: `kernel/src/syscall/process.rs:172-180`
**Problem**: `argv_ptr + i * size_of` could overflow
**Fix**: Use `checked_mul()` and `checked_add()` for safe arithmetic

### Bug 3: INITRAMFS lock held during spawn - FIXED
**Location**: `kernel/src/syscall/process.rs:209-231`
**Problem**: Lock held while calling spawn_from_elf_with_args could cause deadlock
**Fix**: Copy ELF data into Vec, release lock before spawning

## Hypotheses Evaluated

| Hypothesis | Status | Notes |
|------------|--------|-------|
| H1: argv_size overflow | CONFIRMED | Fixed with checked_mul |
| H2: Entry pointer validation | NOT A BUG | Already validated per-entry |
| H3: Empty argc handling | NOT A BUG | argc=0 skips loop correctly |
| H4: Path pointer aliasing | NOT A BUG | Path read before argv |
| H5: Lock held during spawn | CONFIRMED | Fixed by copying ELF data |

## Verification
- [x] Full build succeeds (`cargo xtask build all`)
- [x] All crate tests pass (37 tests)
- [x] No regressions introduced

## Handoff Notes
The SpawnArgs syscall is now:
- Fully enabled and operational (syscall number 15)
- Protected against integer overflow in pointer calculations
- Free of deadlock risk (INITRAMFS lock released before spawn)
- Ready for userspace to use for `cat hello.txt` etc.


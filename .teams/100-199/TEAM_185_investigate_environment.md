# TEAM_185: Investigate Environment Edge Cases

## Bug Report
- **Component**: `args()`, `vars()`, `var()` parsing from stack
- **Task**: Proactively find and fix edge cases/bugs
- **Source**: User requested investigation
- **Status**: ✅ COMPLETED - Fixed 4 bugs

## Files Modified
- `userspace/ulib/src/env.rs` - Environment parsing

## Bugs Found & Fixed

### Bug 1: No bounds check on argc - FIXED
**Location**: `userspace/ulib/src/env.rs:52-56`
**Problem**: Corrupted/huge `argc` could cause reading way past valid stack memory
**Fix**: Added `MAX_ARGC = 4096` limit with `argc.min(MAX_ARGC)`

### Bug 2: envp loop has no upper bound - FIXED
**Location**: `userspace/ulib/src/env.rs:68-77`
**Problem**: If NULL terminator is missing, loop runs forever reading invalid memory
**Fix**: Added `MAX_ENVP = 4096` limit and changed to `while env_idx < MAX_ENVP`

### Bug 3: Vars::next() uses recursion - FIXED
**Location**: `userspace/ulib/src/env.rs:170-181`
**Problem**: Recursive call to skip malformed env vars could stack overflow on many bad entries
**Fix**: Changed from recursion to loop: `loop { ... continue to skip malformed ... }`

### Bug 4: Vars missing size_hint() - FIXED
**Location**: `userspace/ulib/src/env.rs:191-197`
**Problem**: `Vars` iterator didn't implement `size_hint()` unlike `Args`
**Fix**: Added `size_hint()` returning `(0, Some(iter.len()))` to account for skipped entries

## Hypotheses Evaluated

| Hypothesis | Status | Notes |
|------------|--------|-------|
| H1: Stack parsing overflow | CONFIRMED | Fixed with MAX_ARGC/MAX_ENVP bounds |
| H2: Null pointer handling | NOT A BUG | Already checked with `!arg_ptr.is_null()` |
| H3: UTF-8 validation | NOT A BUG | Already skips non-UTF8 with `to_str()` check |
| H4: Double initialization | NOT A BUG | Already checked with `if ARGS.is_some() { return; }` |
| H5: Empty string handling | NOT A BUG | Empty strings are valid, handled correctly |
| H6: Malformed KEY=VALUE | CONFIRMED | Vars::next() now uses loop to skip malformed |

## Known Issues (Not Fixed)
- **static mut warnings**: The code uses `static mut` which generates Rust 2024 warnings. This is a larger refactor (OnceCell/SyncUnsafeCell) that should be done separately.

## Verification
- [x] Full build succeeds (`cargo xtask build all`)
- [x] All crate tests pass (37 tests)
- [x] No regressions introduced

## Handoff Notes
The environment parsing is now hardened with:
- Bounded argc/argv parsing (max 4096 arguments)
- Bounded envp parsing (max 4096 environment variables)
- Stack-safe iteration over malformed environment variables
- Proper size_hint for Vars iterator


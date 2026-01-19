# TEAM_041: Recipe Engine Philosophy Cleanup

## Mission
Remove over-engineered code from recipe engine phases. Keep it simple: lifecycle + UI + helpers only.

## Status: COMPLETED

## Changes Made

### Removed
- `rpm_install()` - Was doing too much magic (parsing cpio output, path validation)
  - Recipes now use `run("rpm2cpio *.rpm | cpio -idmv -D $PREFIX")` for full control
- `validate_path_within_prefix()` - Security theater in the wrong layer
  - If recipes bypass helpers with `run()`, this didn't help anyway
- `install_to_dir()` public API - Internal helper leaked to recipes
  - Renamed to `do_install()` and made private
  - Recipes use `install_bin()`, `install_lib()`, or `run()` instead

### Updated
- Example recipe `systemd-rpm.rhai` - Uses `run()` instead of `rpm_install()`
- Documentation: lib.rs, PHASES.md, README.md - Removed rpm_install references
- Regression test comment - Updated to remove rpm_install reference

### Added
- Doc comments for implicit behavior in acquire.rs, build.rs
- Clear documentation about `last_downloaded`, `current_dir`, env vars

## Philosophy Alignment

**Before:** Engine had magic helpers that did complex logic (rpm_install parsing cpio output)

**After:** Engine provides simple utilities; recipes control complex logic via `run()`

| Concern | Now Handled By |
|---------|----------------|
| RPM extraction | Recipe: `run("rpm2cpio...")` |
| Path validation | Recipe responsibility |
| Custom install paths | Recipe: `run("install -D...")` |

## Files Modified
- `recipe/src/engine/phases/install.rs` - Removed rpm_install, made do_install private
- `recipe/src/engine/phases/mod.rs` - Updated exports
- `recipe/src/engine/mod.rs` - Removed rpm_install registration
- `recipe/src/engine/phases/acquire.rs` - Added doc comments
- `recipe/src/engine/phases/build.rs` - Added doc comments
- `recipe/src/lib.rs` - Updated docs
- `recipe/PHASES.md` - Updated docs
- `recipe/README.md` - Updated docs
- `recipe/examples/systemd-rpm.rhai` - Updated to use run()
- `recipe/tests/regression.rs` - Updated comment

## Line Count Impact
- Removed ~120 lines of over-engineered code
- Added ~40 lines of documentation

## Verification
- All 36 tests pass (19 unit + 17 regression)
- No compiler warnings
- Build succeeds

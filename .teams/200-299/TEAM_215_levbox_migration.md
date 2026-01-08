# TEAM_215: Levbox migration to ulib entry feature

## Objective
Migrate all `levbox` binaries to use the `ulib` entry feature, enabling cleaner code and centralized entry point management.

## Team
- Antigravity (TEAM_215)

## Progress
- [ ] Initial planning and setup
- [ ] Migrate `cat`
- [ ] Migrate `cp`
- [ ] Migrate `ln`
- [ ] Migrate `ls`
- [ ] Migrate `mkdir`
- [ ] Migrate `mv`
- [ ] Migrate `pwd`
- [ ] Migrate `rm`
- [ ] Migrate `rmdir`
- [ ] Migrate `touch`
- [ ] Final verification

## Notes
- `ulib` now provides a `_start` entry point when the `entry` feature is enabled.
- We need to remove the local `_start` and `_start_rust` from each binary.
- Each binary should define `main() -> i32`.

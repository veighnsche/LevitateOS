# TEAM_028: Recipe Repository + Bootstrap Command

## Objective
1. Create `recipes/` repository with base LevitateOS packages
2. Implement `recipe bootstrap` command to install base system

## Status
- [x] Team file created
- [x] Add Bootstrap command to recipe binary
- [x] Create recipes/ directory with base recipes
- [x] Verified with dry-run test

## Files Modified
- `recipe/src/bin/recipe.rs` - Added Bootstrap command + install_with_deps_to_target()
- `recipes/` - New git submodule (git@github.com:LevitateOS/recipes.git) with 9 base .recipe files + README.md + CI workflow

## Decisions
- Using flat structure for recipes (Alpine-style)
- Base packages: base, linux, linux-firmware, systemd, networkmanager, bash, coreutils, util-linux, recipe

## Notes
- Bootstrap creates filesystem hierarchy and installs base packages
- Supports --dry-run and --verbose flags
- Copies recipes to target for self-hosting
- Recipe hashes are placeholders (todo-compute-actual-hash) until real downloads are verified

# TEAM 044: Integrate Recipe Crate into Leviso

## Status: COMPLETE

## Goal
Replace manual `rpm --root` commands in leviso with recipe-based installation using the `levitate-recipe` crate.

## Changes
1. Add `levitate-recipe` dependency to `leviso/Cargo.toml`
2. Generate 82 package recipes in `leviso/recipes/` (one per package in packages.txt)
3. Update `leviso/src/main.rs` to use `RecipeEngine.execute()` instead of manual installation
4. Keep `lib/rocky.rhai` as shared helper (already implemented)

## Files Modified
- `leviso/Cargo.toml` - Added `levitate-recipe = { path = "../recipe" }`
- `leviso/src/main.rs` - RecipeEngine integration, removed `install_package()` and `find_rpm()`
- `leviso/recipes/*.rhai` - 82 package recipes generated (83 total including 00-rocky-iso.rhai)

## Decisions
- Each package gets its own recipe file (recipe engine requires separate files)
- All recipes use identical template (find RPM → extract via rpm2cpio)
- Environment variable `ROCKY_ISO_MOUNT` passes ISO mount point to recipes
- Recipes use `lib/rocky.rhai` helpers: `find_rpm(name)` and `install_rpms()`

## Recipe Template
```rhai
let name = "package-name";
let version = "rocky10";
let installed = false;

fn acquire() {
    import "lib/rocky" as rocky;
    rocky::find_rpm(name);
}

fn install() {
    import "lib/rocky" as rocky;
    rocky::install_rpms();
}
```

## Log
- Added `levitate-recipe` dependency
- Generated 82 package recipes from packages.txt
- Updated main.rs to use RecipeEngine
- Removed unused `install_package()` and `find_rpm()` functions
- Build verified: `cargo build` succeeds with no warnings
- Created root Cargo workspace including leviso, recipe, and recipe/xtask
- Removed workspace declaration from recipe/Cargo.toml (now part of root workspace)

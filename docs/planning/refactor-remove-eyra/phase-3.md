# Phase 3: Migration

## Migration Order

### Step 1: Move libsyscall (No Breaking Changes)

```bash
# Move libsyscall out of eyra workspace
mv crates/userspace/eyra/libsyscall crates/userspace/libsyscall

# Update the Cargo.toml - remove eyra-specific config
# Remove: std = { package = "eyra" } (it doesn't have this anyway)
```

**Files to modify**:
- `crates/userspace/Cargo.toml` - add libsyscall to workspace
- `crates/userspace/libsyscall/Cargo.toml` - update paths if needed

### Step 2: Add xtask sysroot commands

**New file**: `xtask/src/build/sysroot.rs`
```rust
pub fn clone_c_ward() -> Result<()>;
pub fn build_libc() -> Result<()>;
pub fn create_sysroot() -> Result<()>;
pub fn build_sysroot() -> Result<()>;  // Main entry point
```

**New file**: `xtask/src/build/external.rs`

External projects are cloned on-demand (like `npm install` or `go mod download`):

```rust
const COREUTILS_REPO: &str = "https://github.com/uutils/coreutils";
const BRUSH_REPO: &str = "https://github.com/reubeno/brush";

/// Clone coreutils if not present (idempotent)
pub fn clone_coreutils() -> Result<()> {
    let dir = PathBuf::from("toolchain/coreutils");
    if !dir.exists() {
        println!("Cloning uutils/coreutils...");
        Command::new("git")
            .args(["clone", "--depth=1", COREUTILS_REPO, "toolchain/coreutils"])
            .status()?;
    }
    Ok(())
}

/// Build coreutils against our sysroot
pub fn build_coreutils(arch: &str) -> Result<()> {
    clone_coreutils()?;  // Ensure cloned
    // Build with RUSTFLAGS pointing to sysroot...
}

/// Clone brush if not present (idempotent)
pub fn clone_brush() -> Result<()>;

/// Build brush against our sysroot
pub fn build_brush(arch: &str) -> Result<()>;
```

**Key point**: These repos are **gitignored** - they're downloaded at build time, not committed.

**Modify**: `xtask/src/build/commands.rs`
- Add `Sysroot` and `Coreutils` commands
- Update `build_all()` to use new path

### Step 3: Update create_initramfs()

Change binary source paths:
```rust
// OLD:
let coreutils_src = format!(
    "crates/userspace/eyra/coreutils/target/{}/release/coreutils",
    eyra_target
);

// NEW:
let coreutils_src = format!(
    "toolchain/coreutils-out/{}/release/coreutils",
    target
);
```

### Step 4: Move syscall-conformance tests

```bash
# Move conformance tests to tests/
mv crates/userspace/eyra/syscall-conformance tests/syscall_conformance
```

Update test runner to build and run these.

### Step 5: Rename test files

```bash
mv tests/eyra_integration_test.rs tests/userspace_integration_test.rs
mv tests/eyra_regression_tests.rs tests/userspace_regression_tests.rs
```

Update references inside the files.

### Step 6: Delete eyra directory

```bash
# First, deinit the coreutils submodule
git submodule deinit -f crates/userspace/eyra/coreutils
git rm -f crates/userspace/eyra/coreutils

# Then delete the rest
rm -rf crates/userspace/eyra/
```

### Step 7: Update .gitmodules

Remove the eyra coreutils submodule entry.

## Call Site Inventory

### Files Referencing "eyra"

| File | References | Action |
|------|------------|--------|
| `xtask/src/build/commands.rs` | build_eyra(), eyra paths | Replace with new commands |
| `xtask/src/main.rs` | Eyra subcommand | Replace with Sysroot/Coreutils |
| `xtask/src/tests/eyra.rs` | Eyra test functions | Delete or rename |
| `xtask/src/tests/mod.rs` | mod eyra | Remove |
| `tests/eyra_integration_test.rs` | Eyra paths | Rename, update paths |
| `tests/eyra_regression_tests.rs` | Eyra paths | Rename, update paths |
| `CLAUDE.md` | Eyra documentation | Update to c-gull approach |
| `docs/planning/c-gull-migration/` | Already documents new approach | Keep |

### Additional Files to Modify/Delete

TEAM_434 review identified these additional files:

| File | Action |
|------|--------|
| `EYRA_INTEGRATION_COMPLETE.md` | Delete |
| `docs/development/eyra-porting-guide.md` | Delete |
| `scripts/build-eyra.sh` | Delete |
| `.windsurf/workflows/eyra-test-runner.md` | Delete |
| `tests/EYRA_TESTING_README.md` | Delete |
| `tests/eyra_output.txt` | Delete |
| `tests/eyra_behavior_test.sh` | Delete |
| `tests/run_eyra_tests.sh` | Delete |
| `run.sh` | Update (has eyra references) |
| `.github/workflows/release.yml` | Update (has eyra references) |

### Grep Results to Fix

```bash
grep -r "eyra" --include="*.rs" --include="*.toml" --include="*.md" | wc -l
# Currently: ~1626 references across 194 files
# Most are in: archived docs, team files (historical), active code
# Target: 0 references in active code (archives/team files exempt)
```

## Rollback Plan

If migration fails at any step:

1. **Step 1-2 fail**: Just revert commits, no data loss
2. **Step 3-5 fail**: Revert commits, restore from git
3. **Step 6-7 fail**:
   ```bash
   git checkout HEAD~1 -- crates/userspace/eyra/
   git submodule update --init crates/userspace/eyra/coreutils
   ```

Git history preserves everything. No destructive operations until final delete.

## Testing Each Step

| Step | Test Command | Expected Result |
|------|--------------|-----------------|
| 1 | `cargo build -p libsyscall` | Builds successfully |
| 2 | `cargo xtask build sysroot` | Creates sysroot/lib/libc.a |
| 3 | `cargo xtask build all` | Creates initramfs with new binaries |
| 4 | `cargo test syscall_conformance` | Tests pass |
| 5 | `cargo xtask test` | All tests pass |
| 6 | `cargo xtask build all && cargo xtask test` | Full success |
| 7 | `git status` | Clean, no eyra references |

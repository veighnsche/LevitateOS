# Silver vs Gold Golden Files

## Overview

Golden files in LevitateOS tests can be rated as either **Gold** or **Silver**, controlling how test failures are handled during active development.

## The Problem

During active kernel development, running behavior tests with failing golden files creates a frustrating 6-minute workflow:

1. Run `cargo xtask test behavior` (3 minutes)
2. FAILURE: Golden files mismatch, use `--update` flag
3. Run `cargo xtask test behavior --update` (another 3 minutes)
4. Repeat for each change...

This wastes time during periods when behavior is intentionally changing frequently.

## The Solution: Silver Files

**Silver files** auto-update on every test run and always pass, but show you what changed. This eliminates the 6-minute cycle:

1. Run `cargo xtask test behavior` (3 minutes)
2. PASS: SILVER → Golden files updated, diff shown
3. Continue working immediately

When development stabilizes, promote the file back to **Gold** status for strict regression testing.

## Configuration

Golden file ratings are configured in `xtask.toml`:

```toml
[golden_files]
# Gold: Strict mode - must match exactly, test fails on mismatch
"tests/golden_boot.txt" = "gold"

# Silver: Development mode - auto-updates, always passes, shows diff
"tests/golden_boot_x86_64.txt" = "silver"

# Not listed = defaults to "gold"
```

## Rating Semantics

### Gold (Strict Mode)

- **Purpose**: Catch regressions in stable code
- **Behavior**: Test fails if output doesn't match golden file
- **Update**: Requires explicit `--update` flag
- **Use when**: Code is stable and you want to prevent regressions

**Example output:**
```
❌ FAILURE: Behavior REGRESSION detected!

--- Diff ---
- [BOOT] Stage 3: Initialize MMU
+ [BOOT] Stage 3: MMU initialization

💡 TIP: If this change is intentional, run with --update to refresh the golden log.
```

### Silver (Development Mode)

- **Purpose**: Track changes during active development
- **Behavior**: Test always passes, golden file auto-updates
- **Update**: Automatic on every run
- **Use when**: Code is changing frequently and you want to see what's different

**Example output:**
```
🔄 SILVER MODE: Auto-updating golden file...

--- Changes Detected (Golden file updated) ---
- [BOOT] Stage 3: Initialize MMU
+ [BOOT] Stage 3: MMU initialization

Summary: 1 added, 1 removed, 0 changed

✅ VERIFIED: Shell spawned successfully.
✅ VERIFIED: Shell was scheduled.
...
```

If no changes:
```
✅ SILVER MODE: No changes detected.

✅ VERIFIED: Shell spawned successfully.
...
```

## Workflow Recommendations

### During Active Development

1. Set relevant golden files to `silver` in `xtask.toml`
2. Run tests as normal: `cargo xtask test behavior`
3. Review the diffs shown in the output
4. Golden files update automatically
5. Continue development without waiting for double test runs

### Before Merging / Stabilizing

1. Review final behavior changes
2. Promote silver files back to `gold` in `xtask.toml`
3. Run tests one final time to ensure everything passes
4. Commit both code changes and updated `xtask.toml`

### Example Development Cycle

```bash
# Start development on x86_64 boot
vim xtask.toml  # Set golden_boot_x86_64.txt to "silver"

# Make kernel changes
vim crates/kernel/src/main.rs

# Run tests - no need for --update, auto-updates and shows diff
cargo xtask test behavior

# Continue iterating quickly
vim crates/kernel/src/main.rs
cargo xtask test behavior  # Auto-updates again

# When satisfied with changes
vim xtask.toml  # Set back to "gold"
cargo xtask test behavior  # Final verification

# Commit everything
git add xtask.toml tests/golden_boot_x86_64.txt
git commit -m "feat(x86_64): improve boot stage messages"
```

## Supported Tests

The silver/gold system currently supports:

- **Behavior Tests**: `cargo xtask test behavior`
  - `tests/golden_boot.txt` (aarch64)
  - `tests/golden_boot_x86_64.txt` (x86_64)
  - `tests/golden_shutdown.txt` (both architectures)

- **Debug Tools Tests**: `cargo xtask test debug`
  - `tests/golden_debug_regs_aarch64.txt`
  - `tests/golden_debug_regs_x86_64.txt`
  - `tests/golden_debug_mem_aarch64.txt`
  - `tests/golden_debug_mem_x86_64.txt`

## Implementation Details

### Config Loading

The config is loaded from `xtask.toml` at the project root. If the file doesn't exist, all golden files default to `gold`.

```rust
// In test code
let config = XtaskConfig::load()?;
let rating = config.golden_rating("tests/golden_boot.txt");

match rating {
    GoldenRating::Gold => { /* strict comparison */ }
    GoldenRating::Silver => { /* auto-update */ }
}
```

### Diff Output

Silver mode uses a unified diff format:

- Lines starting with `-` were removed (in old golden file)
- Lines starting with `+` were added (in new output)
- Unchanged lines are not shown
- A summary line shows counts: `Summary: X added, Y removed, Z changed`

### Manual Override

Even for gold files, you can still use `--update` to force an update:

```bash
cargo xtask test behavior --update
```

This is useful when you intentionally changed behavior and want to update the golden file without changing `xtask.toml`.

## Best Practices

1. **Don't commit silver ratings to master**: Silver is for your local development. Before merging, convert back to gold.

2. **Use silver sparingly**: Only for files you're actively changing. Too many silver files defeats the purpose of regression testing.

3. **Review diffs carefully**: Even though tests pass, the diff output shows what changed. Read it!

4. **Temporary silver is okay**: It's fine to temporarily set a file to silver, do your work, then set it back to gold before committing.

5. **Document intentional changes**: When promoting silver back to gold, your commit message should explain why the behavior changed.

## Example Scenarios

### Scenario 1: Refactoring Boot Messages

You're cleaning up boot stage messages on x86_64:

```toml
# xtask.toml
[golden_files]
"tests/golden_boot_x86_64.txt" = "silver"  # Temporarily during cleanup
```

Run tests multiple times as you iterate. Each run shows what changed. When done, set back to `"gold"` and commit.

### Scenario 2: Stable Code with Regression

Code is stable, someone introduces a bug:

```toml
# xtask.toml (all files are gold)
[golden_files]
"tests/golden_boot_x86_64.txt" = "gold"
```

Test fails immediately with clear diff showing the regression. Developer must either:
- Fix the bug, or
- Explicitly use `--update` to acknowledge the behavior change

### Scenario 3: Parallel Development

Multiple developers working on different architectures:

```toml
# xtask.toml
[golden_files]
"tests/golden_boot.txt" = "silver"        # Developer A working on aarch64
"tests/golden_boot_x86_64.txt" = "gold"   # x86_64 is stable
```

Each developer sets only their architecture to silver, preventing conflicts.

## FAQ

**Q: Why not always use silver?**  
A: Silver defeats the purpose of regression testing. You want tests to fail when behavior unintentionally changes.

**Q: Can I have per-developer config?**  
A: Yes! Add `xtask.toml` to `.gitignore` (locally) and maintain your own copy. Or use branches with different `xtask.toml` settings.

**Q: What if I forget to set back to gold?**  
A: CI should catch this. Consider adding a check that fails if any files are set to silver in CI builds.

**Q: Can I use wildcards?**  
A: Not currently. Each golden file must be explicitly listed in `xtask.toml`.

**Q: Does this work with `--update`?**  
A: Yes. For gold files, `--update` forces an update. For silver files, `--update` is redundant (they always update).

## See Also

- `xtask.toml` - Configuration file
- `xtask/src/config.rs` - Config parser implementation
- `docs/testing/behavior-inventory.md` - Behavior test documentation
- `.agent/rules/behavior-testing.md` - Testing philosophy

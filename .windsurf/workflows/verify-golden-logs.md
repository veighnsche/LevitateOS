---
description: how to verify and update golden logs for behavior testing
auto_execution_mode: 1
---

# Golden Log Verification Workflow (TEAM_272)

## 0. Purpose
This workflow provides steps to verify that kernel output matches the "golden" reference files required for behavior testing.

## 1. When to Use
Use this workflow when:
1. You modify boot output or logging logic.
2. Behavior tests fail due to output mismatches.
3. You add new features that should be reflected in the boot sequence.

## 2. Verification Steps

### 2.1 Run Behavior Tests
// turbo
1. Run the full behavior test suite:
   ```bash
   cargo xtask test behavior
   ```

### 2.2 Analyze Mismatches
If tests fail, compare the current output with the golden file:
```bash
diff tests/golden_boot.txt tests/golden_boot.txt.new
```

### 2.3 Common Fixes (TEAM_272 Patterns)
- **Noisy Crates:** If external crates (e.g., `virtio_drivers`) add unwanted logs, filter them in `kernel/src/logger.rs`.
- **Level Prefixes:** Ensure `logger.rs` does not print `[INFO]` or `[DEBUG]` prefixes unless the golden file expects them.
- **Duplicate Output:** Check `kernel/src/main.rs` and `kernel/src/init.rs` for redundant `println!` calls across stages.

## 3. Updating Golden Files
If the change is intentional and approved:
1. Overwrite the golden file:
   ```bash
   cp tests/golden_boot.txt.new tests/golden_boot.txt
   ```
2. Commit the change with a clear description of why the output changed.

## 4. Checklist
- [ ] No [LEVEL] prefixes in standard output.
- [ ] No noisy logs from external crates.
- [ ] No duplicate messages across boot stages.
- [ ] Tests pass with `cargo xtask test behavior`.

## 5. Pre-Existing Baseline Failures (TEAM_342)

If tests fail **before** you make any code changes:

### 5.1 Verify It's Pre-Existing
```bash
git status  # Check for uncommitted code changes
git stash   # Temporarily stash changes
cargo xtask test --arch x86_64  # Run test on clean state
git stash pop  # Restore changes
```

### 5.2 Determine Cause
- **Recent refactors:** Golden log may not have been updated after merges
- **Environment differences:** Memory layout, addresses may vary
- **New features:** Added logging that wasn't captured

### 5.3 Resolution Options
1. **Update golden log** (if current behavior is correct):
   ```bash
   cargo xtask test --arch x86_64 --update
   ```
2. **Investigate regression** (if old behavior should be restored)
3. **Ask user** for guidance if unclear

### 5.4 Rule 4 Reminder
Per global rules: Never modify baseline data unless USER explicitly approves.
If in doubt, ask before updating golden files.

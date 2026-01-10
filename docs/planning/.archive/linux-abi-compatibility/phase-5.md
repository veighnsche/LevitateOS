# Phase 5: Cleanup, Regression Protection, and Handoff

**TEAM_339** | Linux ABI Compatibility Bugfix

## Overview

Final verification, documentation, and handoff.

## Cleanup Tasks

### Remove Dead Code
- Remove old `copy_user_string(ptr, len, buf)` if replaced
- Remove duplicate errno definitions
- Remove any temporary debug logging

### Code Quality
- Run clippy on all changed files
- Fix any new warnings
- Ensure consistent style

## Regression Protection

### New Tests to Add

| Test | File | Purpose |
|------|------|---------|
| `test_linux_abi_openat` | `tests/syscall_abi.rs` | Verify openat matches Linux |
| `test_linux_abi_paths` | `tests/syscall_abi.rs` | Verify all path syscalls |
| `test_stat_struct_size` | `tests/struct_layout.rs` | Compile-time size check |
| `test_termios_struct_size` | `tests/struct_layout.rs` | Compile-time size check |

### Golden File Updates
If behavior tests change output, update golden files.

### Documentation Updates

| Doc | Update |
|-----|--------|
| `docs/ARCHITECTURE.md` | Document Linux ABI compatibility |
| `README.md` | Note Linux binary compatibility |
| `crates/userspace/libsyscall/README.md` | Update API docs |

## Handoff Checklist

- [ ] All tests pass (x86_64 and aarch64)
- [ ] No new compiler warnings
- [ ] Documentation updated
- [ ] Question file resolved/archived
- [ ] Team file updated with final status

---

## Steps

### Step 1: Final Test Sweep

**Tasks:**
1. Run full test suite on both architectures
2. Run behavior tests
3. Manual smoke test of shell operations

**Output:** Test report

### Step 2: Documentation Updates

**Tasks:**
1. Update ARCHITECTURE.md
2. Update libsyscall README
3. Archive question file

**Output:** Updated docs

### Step 3: Handoff Notes

**Tasks:**
1. Update team file with completion status
2. Document any remaining edge cases
3. Note future work (e.g., more syscalls to add)

**Output:** Complete team file

---

## Exit Criteria

- [ ] All tests pass
- [ ] Documentation complete
- [ ] Team file shows completed status
- [ ] Question file archived
- [ ] Ready for production use

# TEAM_434: Review Plan - Remove Eyra Refactor

## Objective

Review the TEAM_433 plan to remove Eyra and externalize userspace dependencies.

## Status: COMPLETE

## Plan Location

`docs/planning/refactor-remove-eyra/` (5 phases)

## Review Summary

**Overall Assessment**: APPROVED with minor refinements needed

The plan is well-structured, follows project rules, and correctly identifies the migration path from Eyra to c-gull sysroot. The c-gull infrastructure already exists (TEAM_430), making this primarily a deletion/cleanup task.

### Strengths

1. **Clear externalization model** - External deps cloned at build time, gitignored (like npm/go modules)
2. **No compatibility shims** - Clean break per Rule 5
3. **Phased approach** - Safe incremental migration with rollback points
4. **Documentation updates planned** - CLAUDE.md, README.md updates included

### Issues Found

#### Critical (blocks work)

None.

#### Important (quality)

1. **Scope underestimated** - Plan says ~50 "eyra" references, actual count is **1626 references across 194 files**. Most are in:
   - Archived planning docs (can ignore)
   - Team files (historical, keep as-is)
   - Active code paths (need updating)

2. **Missing files in Phase 3 inventory**:
   - `EYRA_INTEGRATION_COMPLETE.md` (root) - needs deletion
   - `docs/development/eyra-porting-guide.md` - needs deletion
   - `scripts/build-eyra.sh` - needs deletion
   - `.windsurf/workflows/eyra-test-runner.md` - needs deletion
   - `tests/EYRA_TESTING_README.md` - needs deletion/rename

3. **Test files not fully enumerated** - Phase 3 only mentions `eyra_integration_test.rs` and `eyra_regression_tests.rs`, but also:
   - `tests/eyra_output.txt`
   - `tests/eyra_behavior_test.sh`
   - `tests/run_eyra_tests.sh`

4. **Kernel references** - Several kernel files have "eyra" comments:
   - `crates/kernel/syscall/src/sync.rs` (2)
   - `crates/kernel/syscall/src/time.rs` (1)
   - `crates/kernel/mm/src/user/layout.rs` (2)
   - These are historical comments, not functional dependencies

#### Minor (polish)

1. **Golden file** - `tests/golden_boot_x86_64.txt` has "eyra" references that may need updating

## Verified Claims

| Claim | Status | Notes |
|-------|--------|-------|
| c-gull sysroot exists | VERIFIED | `toolchain/sysroot/lib/libc.a` exists (ar archive) |
| libsyscall is pure no_std | VERIFIED | Cargo.toml has no eyra dependency |
| coreutils submodule is forked | VERIFIED | `.gitmodules` points to `LevitateOS/coreutils` fork |
| unmodified-coreutils cloned | VERIFIED | `toolchain/unmodified-coreutils/` exists |
| build-coreutils.sh works | VERIFIED | Script exists with correct RUSTFLAGS |

## Architecture Alignment

- **Rule 0 (Quality)**: Plan removes technical debt
- **Rule 4 (Silence is Golden)**: No logging changes needed
- **Rule 5 (No shims)**: Explicitly forbids compatibility wrappers
- **Rule 6 (Cleanup phase)**: Phase 4 covers dead code removal
- **Rule 7 (File sizes)**: Plan targets <500 lines per module

## Rules Compliance Checklist

- [x] Rule 0: No shortcuts (clean break)
- [x] Rule 4: Tests/baselines mentioned (Phase 5)
- [x] Rule 5: No compatibility hacks
- [x] Rule 6: Cleanup phase exists (Phase 4)
- [x] Rule 7: Structure well-scoped (commands.rs split planned)
- [ ] Rule 10: Handoff checklist exists (Phase 5 has success criteria, needs explicit handoff checklist)

## Refinements to Apply

### Phase 3 - Add missing files to call site inventory:

```markdown
## Additional Files to Modify/Delete

| File | Action |
|------|--------|
| `EYRA_INTEGRATION_COMPLETE.md` | Delete |
| `docs/development/eyra-porting-guide.md` | Delete |
| `scripts/build-eyra.sh` | Delete |
| `.windsurf/workflows/eyra-test-runner.md` | Delete |
| `tests/EYRA_TESTING_README.md` | Delete |
| `tests/eyra_output.txt` | Delete |
| `tests/eyra_behavior_test.sh` | Delete/rename |
| `tests/run_eyra_tests.sh` | Delete |
| `run.sh` | Update (has eyra references) |
| `.github/workflows/release.yml` | Update (has eyra references) |
```

### Phase 4 - Add kernel comment cleanup:

Kernel files with historical "eyra" comments should be cleaned up:
- These are informational, not functional
- Low priority but part of "no dead code"

### Phase 5 - Add explicit handoff checklist:

```markdown
## Handoff Checklist

- [ ] Project builds: `cargo xtask build all`
- [ ] All tests pass: `cargo xtask test`
- [ ] No "eyra" in active code (archive docs exempt)
- [ ] Team file has complete progress log
- [ ] Remaining TODOs documented
- [ ] Gotchas added to docs/GOTCHAS.md
```

## Questions Audit

### Answered Questions Reflected

- Q1-Q7 from TEAM_349 (Eyra integration prerequisites) - These syscalls are now in kernel, plan doesn't affect them
- TEAM_391 (brush shell) - Plan includes brush as external dependency

### Open Questions

None blocking - all requirements clarified in plan.

## Progress Log

### Session 1 (2026-01-11)
- Created review team file
- Read all 5 plan phases
- Checked archived questions related to Eyra
- Verified c-gull infrastructure exists
- Counted actual "eyra" references (1626 across 194 files)
- Identified missing files in migration inventory
- Verified claims about existing infrastructure
- Completed review with refinements

## Recommendation

**PROCEED with minor updates** to Phase 3 (add missing files) and Phase 5 (add handoff checklist).

The plan is sound, infrastructure is ready, and the migration is well-defined. The main work is deletion and xtask updates, which is lower risk than building new functionality.

# Phase 4: Cleanup and Hardening

## Objective
Final cleanup, documentation, and ensure the refactor is complete.

## Tasks

### Step 1: Update .gitignore
Ensure stale per-utility artifacts can't be recreated:
```gitignore
# In crates/userspace/eyra/.gitignore
# Only allow workspace-level target
/*/target/
/*/Cargo.lock
/*/.cargo/
```

### Step 2: Update Documentation
- Update TOOLCHAIN_MANAGEMENT.md with Eyra workspace structure
- Document the testing strategy in docs/TESTING.md

### Step 3: Keep eyra-hello as Minimal Test Binary
The `eyra-hello` utility serves as a minimal Eyra sanity check. It's already in the workspace and should remain as a quick verification that Eyra builds work.

### Step 4: Final Verification
```bash
# Full rebuild from scratch
rm -rf crates/userspace/eyra/target
./run-test.sh

# Verify only workspace target exists
find crates/userspace/eyra -name "target" -type d
# Should only show: crates/userspace/eyra/target
```

## Success Criteria
- [ ] .gitignore prevents future stale artifacts
- [ ] Documentation updated
- [ ] Clean workspace structure
- [ ] All tests pass from clean state

## Handoff Checklist
- [ ] `./run-test.sh` passes
- [ ] Only `crates/userspace/eyra/target/` exists (no per-utility targets)
- [ ] Documentation reflects new structure
- [ ] Team file updated with completion status

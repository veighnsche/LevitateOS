# Phase 5: Hardening

## Final Verification

### Build Verification

```bash
# Both architectures must compile
cargo xtask build kernel --arch aarch64
cargo xtask build kernel --arch x86_64

# Run clippy for warnings
cargo clippy --package levitate-kernel --target aarch64-unknown-none
cargo clippy --package levitate-kernel --target x86_64-unknown-none
```

### Behavior Tests

```bash
# Run behavior tests to verify syscall behavior unchanged
cargo xtask test behavior

# If any golden files changed unexpectedly, investigate before updating
```

### Runtime Verification

```bash
# Boot and verify basic process operations work
cargo xtask run

# Test spawning processes
> /bin/echo hello
> /bin/ls

# Test process hierarchy
# (getpid/getppid should work correctly)
```

## Documentation Updates

### Module-Level Documentation

Each new file should have a module doc comment:

```rust
//! Process lifecycle syscalls.
//!
//! TEAM_417: Extracted from monolithic process.rs for maintainability.
//! See `docs/planning/refactor-process-syscalls/` for refactor details.
```

### Update ARCHITECTURE.md if needed

If the syscall module structure is documented in `docs/ARCHITECTURE.md`, update it to reflect the new `process/` subdirectory structure.

### Update behavior-inventory.md

If any behavior IDs reference `process.rs`, update to reference the new module location.

## Handoff Notes

### What Changed

1. **File structure**: Single `process.rs` (1090 lines) split into 7 focused modules totaling ~880 lines
2. **No API changes**: All public syscall functions maintain identical signatures
3. **No behavior changes**: Syscalls work exactly as before

### New Module Locations

| Syscall | Old Location | New Location |
|---------|--------------|--------------|
| sys_exit | process.rs:141 | process/lifecycle.rs |
| sys_spawn | process.rs:182 | process/lifecycle.rs |
| sys_clone | process.rs:451 | process/thread.rs |
| sys_getuid | process.rs:587 | process/identity.rs |
| sys_arch_prctl | process.rs:636 | process/arch_prctl.rs |
| sys_setpgid | process.rs:744 | process/groups.rs |
| sys_getrusage | process.rs:980 | process/resources.rs |

### Future Considerations

1. **sys_exec** is still a stub - when implementing, work in `lifecycle.rs`
2. **sys_clone** fork-style not supported - when adding, extend `thread.rs`
3. **Resource limits** aren't enforced - when implementing, extend `resources.rs`
4. **arch_prctl.rs** is x86_64-only - aarch64 equivalent would be separate

### Testing Recommendations

Before this refactor is considered complete:

1. Boot LevitateOS on both architectures
2. Spawn at least one process (e.g., `/bin/echo`)
3. Verify shell can run multiple commands
4. Verify `uname` returns correct values

## Completion Checklist

- [ ] All 7 modules created and populated
- [ ] Old `process.rs` deleted
- [ ] Both architectures compile
- [ ] No new clippy warnings introduced
- [ ] Behavior tests pass
- [ ] Runtime smoke test passes
- [ ] TEAM file created: `.teams/TEAM_417_refactor_process_syscalls.md`
- [ ] Plan directory complete: `docs/planning/refactor-process-syscalls/`

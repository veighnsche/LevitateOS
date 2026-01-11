# Phase 5: Hardening

## Final Verification

### Build Verification

```bash
# Clean build from scratch
cargo clean
rm -rf toolchain/sysroot toolchain/c-ward toolchain/coreutils-out

# Full build
cargo xtask build all

# Verify outputs exist
ls toolchain/sysroot/lib/libc.a
ls toolchain/coreutils-out/*/release/coreutils
ls initrd_root/coreutils
```

### Test Suite

```bash
# All tests must pass
cargo xtask test

# Specific test categories
cargo xtask test unit
cargo xtask test behavior
cargo xtask test regress
```

### Runtime Verification

```bash
# Boot and test in QEMU
cargo xtask run

# In shell, verify commands work:
# $ echo hello
# $ cat /hello.txt
# $ pwd
# $ ls
```

### aarch64 Parity

```bash
# Build for aarch64
cargo xtask --arch aarch64 build all

# Run (if QEMU aarch64 works)
cargo xtask --arch aarch64 run
```

## Documentation Updates

### Updated Files

| File | Status | Changes |
|------|--------|---------|
| `CLAUDE.md` | Updated | c-gull section replaces Eyra |
| `README.md` | Updated | Build instructions |
| `docs/planning/c-gull-migration/BUILD_INSTRUCTIONS.md` | Kept | Reference doc |
| `docs/planning/refactor-remove-eyra/` | Created | This plan |

### Removed Documentation

| File | Reason |
|------|--------|
| `crates/userspace/eyra/README.md` | Deleted with eyra/ |
| `crates/userspace/eyra/BEHAVIOR_INVENTORY.md` | Deleted |
| `crates/userspace/eyra/NOSTARTFILES_README.md` | Deleted |
| `crates/userspace/eyra/TEST_SUMMARY.md` | Deleted |

## Handoff Notes

### What Changed

1. **Removed**: `crates/userspace/eyra/` directory entirely
2. **Added**: `toolchain/` sysroot build infrastructure
3. **Updated**: xtask build commands
4. **Renamed**: Test files from `eyra_*` to `userspace_*`
5. **Moved**: `libsyscall` to `crates/userspace/libsyscall`

### New Build Flow

```
cargo xtask build all
  ├── build_sysroot()       # toolchain/sysroot/lib/libc.a
  ├── build_coreutils()     # Unmodified uutils → toolchain/coreutils-out/
  ├── build_userspace()     # init, shell (no-std)
  ├── create_initramfs()    # Bundle everything
  └── build_kernel()        # Kernel binary
```

### Key Files

| Purpose | Location |
|---------|----------|
| libc source | `toolchain/c-ward/` (cloned) |
| libc wrapper | `toolchain/libc-levitateos/` |
| Pre-built sysroot | `toolchain/sysroot/lib/` |
| Coreutils output | `toolchain/coreutils-out/` |
| Build sysroot | `cargo xtask build sysroot` |
| Build coreutils | `cargo xtask build coreutils` |

### Known Limitations

1. **aarch64 not fully tested** - sysroot builds, but may need linker adjustments
2. **Missing libc functions** - `getpwuid`, `getgrgid`, `nl_langinfo` not in c-gull
3. **Limited utilities** - Only basic coreutils work (cat, echo, pwd, etc.)

### Future Work

1. **Add brush shell** - Build unmodified brush against sysroot
2. **Implement missing libc** - Contribute to c-gull upstream
3. **Full coreutils** - Enable all utilities as libc grows
4. **Dynamic linking** - Eventually support ld.so for true Linux compat

## Success Criteria Checklist

- [ ] No "eyra" references in source code
- [ ] No "eyra" references in Cargo.toml files
- [ ] `crates/userspace/eyra/` does not exist
- [ ] `cargo xtask build all` succeeds for x86_64
- [ ] `cargo xtask build all --arch aarch64` succeeds
- [ ] `cargo xtask test` passes all tests
- [ ] Kernel boots and shell works in QEMU
- [ ] Coreutils commands work (cat, echo, pwd)
- [ ] Documentation updated
- [ ] Team file completed

## Handoff Checklist

Before marking complete:

- [ ] Project builds: `cargo xtask build all`
- [ ] All tests pass: `cargo xtask test`
- [ ] No "eyra" in active code (archive docs and team files exempt)
- [ ] Team file has complete progress log
- [ ] Remaining TODOs documented in team file
- [ ] Any gotchas added to `docs/GOTCHAS.md`
- [ ] CLAUDE.md updated with c-gull instructions

## Team Sign-Off

```
TEAM_433: Refactor Remove Eyra
Status: [PLANNING | IN_PROGRESS | COMPLETE]
Date: YYYY-MM-DD
```

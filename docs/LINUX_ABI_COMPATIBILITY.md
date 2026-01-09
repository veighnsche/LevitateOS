# Linux ABI Compatibility

**TEAM_339** | 2026-01-09

## Summary

LevitateOS uses Linux syscall **numbers** but has **custom syscall signatures**. This means it is NOT Linux binary-compatible.

## Key Differences

### Syscall Signatures

LevitateOS path syscalls use length-counted strings `(ptr, len)` instead of Linux's null-terminated strings.

| Syscall | Linux Signature | LevitateOS Signature |
|---------|-----------------|----------------------|
| `openat` | `(dirfd, pathname, flags, mode)` | `(path, path_len, flags)` |
| `mkdirat` | `(dirfd, pathname, mode)` | `(dfd, path, path_len, mode)` |
| `unlinkat` | `(dirfd, pathname, flags)` | `(dfd, path, path_len, flags)` |

### Architecture-Specific Issues

- `__NR_pause` is hardcoded as 34 (x86_64 only)
- aarch64 Linux doesn't have `pause` - uses `ppoll(NULL, 0, NULL, NULL)`

### Struct Layouts

Kernel defines custom `Stat` and `Termios` structs that may differ from `linux_raw_sys` definitions used in userspace.

## Decision: Full Linux ABI Compatibility

**User chose:** Option A (2026-01-09)

The project will refactor all syscalls to match Linux ABI exactly. See:
- Plan: `docs/planning/linux-abi-compatibility/`
- Question: `docs/questions/TEAM_339_linux_abi_compatibility_decision.md`

## For Future Teams

### When Adding New Syscalls

1. **Check Linux man pages** for exact signature
2. **Use null-terminated strings** for paths (not length-counted)
3. **Support `AT_FDCWD`** (-100) for `*at()` syscalls
4. **Use `linux_raw_sys`** types for structs when possible

### Key Files

| Area | Location |
|------|----------|
| Kernel syscall handlers | `crates/kernel/src/syscall/` |
| Userspace wrappers | `crates/userspace/libsyscall/src/` |
| Syscall numbers | `crates/userspace/libsyscall/src/sysno.rs` |
| Arch-specific code | `crates/kernel/src/arch/*/mod.rs` |

### Verification

After modifying syscalls:
1. Run `cargo xtask test --arch x86_64`
2. Run `cargo xtask test --arch aarch64`
3. Verify userspace apps still work

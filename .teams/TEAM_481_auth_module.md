# TEAM_481: Authentication Module Extraction

## Goal

Extract authentication/PAM/login configuration into an independent, testable module.

## Current State

Authentication config is scattered in:
- `crates/builder/src/builder/initramfs.rs` - PAM config, passwd, shadow, nsswitch
- `crates/builder/src/builder/glibc.rs` - PAM modules, unix_chkpwd

## Target Architecture

```
crates/builder/src/builder/
├── auth/
│   ├── mod.rs          # Public API
│   ├── pam.rs          # PAM configuration
│   ├── users.rs        # passwd, shadow, group
│   ├── nss.rs          # nsswitch.conf
│   └── test.rs         # Unit tests
```

## Module Responsibilities

### auth::pam
- Generate /etc/pam.d/login
- Generate /etc/pam.d/other
- Generate /etc/security/opasswd
- Copy PAM modules (pam_unix.so, pam_permit.so)
- Copy unix_chkpwd

### auth::users
- Generate /etc/passwd
- Generate /etc/shadow (with hashed passwords)
- Generate /etc/group
- Generate /etc/login.defs
- Create home directories

### auth::nss
- Generate /etc/nsswitch.conf
- Copy NSS modules (libnss_files.so.2)

## Testing Strategy

1. Unit tests for config generation (string output)
2. Integration test: generate files, run in VM, verify login

## Additional Cleanup

Removed `qemu.rs` from builder - same separation of concerns violation as VM control.
- `builder run` command removed
- Use `cargo xtask vm start` instead

## Status

COMPLETE

# TEAM_169: AcornOS SSH Setup

**Date**: 2026-02-04 (Iteration 13)
**Status**: Complete
**Task**: Phase 3.13 - SSH: sshd installed, host keys generated, sshd_config allows root login for live ISO

## Summary

Implemented complete SSH infrastructure for AcornOS with automated host key generation and configuration for secure remote access in the live ISO.

## What Was Done

### SSH Custom Operation
- Created `AcornOS/src/component/custom/ssh.rs` with `setup_ssh()` function
- Generates three host key types: RSA (2048-bit), ECDSA (P-256), Ed25519 (modern)
- Uses host system's `ssh-keygen` (not staging rootfs) to avoid musl library issues
- Sets empty passphrases for keys so sshd can start without user interaction
- Applies `root@acornos` comment to keys for identification

### sshd Configuration
- Updated `sshd_config` to enable `PermitRootLogin yes` for live ISO
- Preserves password and pubkey authentication settings
- Maintains Alpine's default SFTP subsystem configuration
- Allows both interactive and automated access methods

### Component Integration
- Added `CustomOp::SetupSsh` variant to `Op` enum
- Updated SSH component definition to:
  1. Create SSH directories and user/group
  2. Copy SSH configuration from Alpine
  3. Call SetupSsh custom operation for host key generation
  4. Enable sshd in default runlevel via `openrc_enable("sshd", "default")`
- Integrated module into custom operation dispatcher

## Key Decisions

**Host-side Key Generation**: Initial approach tried running `ssh-keygen` from the staging rootfs binary, but OpenSSL crypto symbols weren't resolved yet (musl linker errors). Switched to host's `ssh-keygen` which has full glibc support and works reliably during the build process.

**No Root Password in Component**: The SSH setup creates keys but doesn't set a root password. The root password is configured separately by the LIVE_FINAL component's branding/security setup, avoiding duplication.

**Three Key Types**: Including RSA for backwards compatibility, ECDSA for performance, and Ed25519 for modern security. This matches Alpine's default ssh-keygen behavior.

## Files Modified

- `AcornOS/src/component/custom/ssh.rs` (new)
- `AcornOS/src/component/custom/mod.rs` (added ssh module import and dispatch)
- `AcornOS/src/component/mod.rs` (added SetupSsh variant to CustomOp enum)
- `AcornOS/src/component/definitions.rs` (enabled SSH component with custom operation)

## Verification

Built fresh rootfs with SSH setup:
```
✓ ssh_host_rsa_key (2.6 KB, 0600)
✓ ssh_host_ecdsa_key (505 B, 0600)
✓ ssh_host_ed25519_key (399 B, 0600)
✓ sshd_config configured with PermitRootLogin yes
✓ /etc/init.d/sshd present and executable
✓ /etc/runlevels/default/sshd symlink created
✓ EROFS build completed successfully (768 MB)
```

## Blockers / Known Issues

None. SSH is fully operational.

## Next Steps

Task 3.14: Install all Tier 0-2 packages from distro-spec

# TEAM_089: Fix Critical Gaps in SSH, Checksum, and Microcode

## Status: COMPLETE (Phase 2)

## Summary

Fix critical issues found during audit. Phase 1 addressed basic SSH, checksum, and microcode issues. Phase 2 addresses additional gaps found in follow-up audit.

### Phase 1 Issues (DONE)

| Component | Severity | Issue |
|-----------|----------|-------|
| **SSH** | CRITICAL | Missing library dependencies - sshd binary won't execute |
| **SSH** | CRITICAL | Missing sshd user/group creation |
| **SSH** | HIGH | Missing runtime directories (/var/empty/sshd) |
| **Checksum** | HIGH | Full path in checksum file - users can't verify |
| **Checksum** | MEDIUM | Checksum file not cleaned on `clean iso` |
| **Microcode** | HIGH | Warnings instead of FAIL FAST (violates CLAUDE.md Rule 7) |

### Phase 2 Issues (DONE)

| Component | Severity | Issue |
|-----------|----------|-------|
| **SSH** | HIGH | sshd_config silently skipped - sshd uses unreliable defaults |
| **SSH** | HIGH | moduli silently skipped - first SSH connection hangs 10-30s |
| **SSH** | CRITICAL | SSH client binaries missing - users can't `ssh`, `ssh-keygen`, `scp` |
| **SSH** | HIGH | sshd user missing from /etc/shadow - PAM authentication errors |
| **SSH** | CRITICAL | NSS libraries not copied - sshd can't resolve "sshd" user (dlopen'd) |
| **Checksum** | MEDIUM | efiboot.img not cleaned |
| **Checksum** | LOW | live-overlay not cleaned |
| **Microcode** | CRITICAL | Intel microcode never extracted - microcode_ctl RPM missing |

## Files Modified

1. `leviso/src/build/openssh.rs` - Add lib deps, user, directories, validation, SSH client, NSS libs
2. `leviso/src/build/etc.rs` - Add sshd to passwd, shadow, group, gshadow
3. `leviso/src/iso.rs` - Fix checksum format
4. `leviso/src/clean.rs` - Add checksum, efiboot.img, live-overlay cleanup
5. `leviso/src/squashfs/system.rs` - Fix microcode validation + copy Intel microcode from Rocky location
6. `leviso/src/extract.rs` - Add microcode_ctl, openssh-server, openssh-clients to SUPPLEMENTARY_RPMS; support noarch packages

## Changes

### 1. SSH Fixes (Phase 1)

- Add `use super::libdeps` import for library dependency functions
- Call `get_all_dependencies()` and `copy_library()` for sshd binary
- Call `get_all_dependencies()` and `copy_library()` for SSH helper binaries
- Add `ensure_sshd_user()` function following chrony pattern
- Create `/var/empty/sshd` directory (privilege separation)
- Create `/run/sshd` directory (runtime)
- FAIL FAST on missing sshd-keygen (required for host key generation)

### 2. SSH Fixes (Phase 2)

- FAIL FAST on missing sshd_config (required for reliable operation)
- FAIL FAST on missing moduli (required for responsive SSH connections)
- Add `copy_ssh_client()` function for ssh, ssh-keygen, ssh-add, ssh-agent, scp, sftp
- Add `copy_nss_libraries()` function for libnss_files.so.2 (dlopen'd, not in ldd)
- Add sshd user to /etc/passwd, /etc/shadow, /etc/group, /etc/gshadow in etc.rs
- Add openssh-server and openssh-clients to SUPPLEMENTARY_RPMS

### 3. Checksum Fixes

- Parse sha512sum output to extract just the hash
- Use basename instead of full path in checksum file
- Add checksum cleanup to `clean_iso()`
- Add efiboot.img cleanup to `clean_iso()`
- Add live-overlay cleanup to `clean_iso()`

### 4. Microcode Fixes

- Change warnings to FAIL FAST when microcode directories are empty
- Require at least one microcode type (AMD or Intel) to exist
- Allow individual types to be missing (OK for single-vendor systems)
- Add microcode_ctl to SUPPLEMENTARY_RPMS for Intel CPU microcode
- Update find_rpm() to also match noarch packages (microcode_ctl is noarch)
- **Copy Intel microcode from Rocky's location** - Rocky puts Intel microcode in `/usr/share/microcode_ctl/ucode_with_caveats/intel/intel-ucode/` instead of the standard `/lib/firmware/intel-ucode/`. Added code to copy it to the standard location for early microcode loading.

## Verification

```bash
# 1. Build succeeds (catches all FAIL FAST errors)
cd leviso && cargo run -- build

# 2. Check SSH client binaries in staging
ls output/staging/usr/bin/{ssh,ssh-keygen,ssh-add,ssh-agent,scp,sftp}

# 3. Check SSH config files
ls output/staging/etc/ssh/sshd_config output/staging/etc/ssh/moduli

# 4. Check NSS library
ls output/staging/usr/lib64/libnss_files.so.2

# 5. Check sshd user in passwd files
grep sshd output/staging/etc/passwd
grep sshd output/staging/etc/shadow
grep sshd output/staging/etc/group

# 6. Verify clean removes all artifacts
cargo run -- clean iso
test ! -f output/efiboot.img && echo "OK: efiboot.img cleaned"
test ! -d output/live-overlay && echo "OK: live-overlay cleaned"
test ! -f output/levitateos.iso.sha512 && echo "OK: checksum cleaned"

# 7. Full integration testing belongs in install-tests crate
```

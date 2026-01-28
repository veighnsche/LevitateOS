# TEAM_141: qcow2 Testing Infrastructure

**Created:** 2026-01-28
**Status:** Complete

## Objective

Add comprehensive testing for qcow2 VM images with two complementary approaches:
1. **Static verification** (fsdbg-style) - Check that all required files and configurations exist
2. **Behavioral testing** (rootfs-tests renewal) - Verify the image actually boots and services work

## Approach

### Part 1: Static Verification - fsdbg qcow2 checklist

Extend `testing/fsdbg/` with a new checklist for qcow2 images:
- Mount qcow2 via qemu-nbd (read-only)
- Verify boot loader, kernel, initramfs, fstab
- Verify system config and enabled services
- Check security (empty root password, no SSH keys)

### Part 2: Behavioral Testing - Renew rootfs-tests

Complete renewal of `testing/rootfs-tests/` from systemd-nspawn to QEMU-based testing:
- Use recqemu for VM construction and serial I/O
- Boot real VMs and verify services work
- Test login, filesystem, security

## Implementation Log

### 2026-01-28: Implementation complete

**Part 1: Static Verification (fsdbg)**
- Created `testing/fsdbg/src/checklist/qcow2.rs` with static verification checks:
  - Boot loader: systemd-boot EFI, loader.conf, boot entries
  - Kernel/initramfs: vmlinuz-*, initramfs-*.img
  - Filesystem: fstab with / and /boot entries
  - System config: hostname, machine-id (empty), shadow (empty root password)
  - Services: NetworkManager, sshd, chronyd enabled
  - Security: SSH host keys absent (regenerate on first boot)
- Updated `mod.rs` with Qcow2 variant
- Updated `main.rs` with qemu-nbd mounting logic and cleanup guard

**Part 2: Behavioral Testing (rootfs-tests)**
- Complete renewal from systemd-nspawn to QEMU-based testing
- Created `QcowTestHarness` using recqemu for VM control
- Test files:
  - `tests/boot.rs` - Boot, login, no panic, no emergency mode
  - `tests/services.rs` - NetworkManager, sshd, chronyd, no failed services
  - `tests/filesystem.rs` - Root/boot mounts, writable, fstab entries
  - `tests/security.rs` - SSH keys regenerated, machine-id, permissions

## Files Changed

- `testing/fsdbg/src/checklist/qcow2.rs` - NEW: qcow2 checklist
- `testing/fsdbg/src/checklist/mod.rs` - Add Qcow2 variant
- `testing/fsdbg/src/main.rs` - Add qcow2 verification support via qemu-nbd
- `testing/fsdbg/Cargo.toml` - Add tempfile dependency
- `testing/rootfs-tests/Cargo.toml` - Replace deps with recqemu
- `testing/rootfs-tests/src/lib.rs` - NEW: QcowTestHarness
- `testing/rootfs-tests/tests/boot.rs` - NEW: Boot tests (+ service failure detection)
- `testing/rootfs-tests/tests/services.rs` - NEW: Service tests
- `testing/rootfs-tests/tests/filesystem.rs` - NEW: Filesystem tests
- `testing/rootfs-tests/tests/network.rs` - NEW: Network tests
- `testing/rootfs-tests/tests/security.rs` - NEW: Security tests
- `testing/rootfs-tests/CLAUDE.md` - Updated documentation
- Removed old files: container.rs, main.rs, tests/*.rs (old)

## Bug Fixes Applied

After initial implementation, reviewed for bugs and gaps:

1. **Permission handling in qcow2 verification**: Added graceful handling for permission errors when reading files like /etc/shadow. Reports "permission denied (run with sudo)" instead of generic error.

2. **Fixed bind-mount panic**: Changed `status.unwrap()` after `status.is_err()` check to use proper `if let` matching.

3. **Added missing static checks**:
   - /etc/passwd (root entry exists)
   - /etc/group (root group exists)
   - /etc/os-release (NAME and ID fields)
   - /etc/locale.conf (optional)
   - Boot entry kernel/initramfs path validation
   - Boot entry options root= parameter check

4. **Added missing behavioral tests**:
   - `tests/network.rs` - Network interface, loopback, DHCP, localhost resolution
   - Boot service failure detection (`test_no_boot_service_failures`)
   - systemd-analyze verification

5. **Added libc dependency** for getuid() check to warn about sudo requirement.

## Integration

### Leviso Build Integration

After `cargo run -p leviso -- build qcow2`:
- `artifact::verify_qcow2()` is called automatically
- If running as root, mounts qcow2 via NBD and runs fsdbg checklist
- If not root, skips NBD verification with a warning

### Rootfs-tests Preflight

All rootfs-tests now run preflight checks before booting:
- `tests/common/mod.rs` provides `ensure_preflight()` and `qcow2_path()`
- Preflight runs fsdbg verification on the qcow2
- Catches obvious problems before spending time booting VMs

## Usage

```bash
# Build qcow2 with verification (as root for full verification)
sudo cargo run -p leviso -- build qcow2

# Static verification only
sudo fsdbg verify output/levitateos.qcow2 --type qcow2

# Behavioral tests (requires KVM, runs preflight automatically)
cargo test -p rootfs-tests --ignored
```

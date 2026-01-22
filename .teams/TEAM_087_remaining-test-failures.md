# TEAM_087: Remaining Install-Test Failures

## Status: Open

## Context

TEAM_086 fixed the post-reboot boot failure. The system now installs and boots from disk successfully. However, 2 of 6 post-reboot verification steps still fail.

## Current Test Results

| Step | Name | Status | Notes |
|------|------|--------|-------|
| 1-18 | Installation | PASS | All installation steps work |
| 19 | Verify Systemd Boot | PASS | System boots, systemd runs |
| 20 | Verify Hostname | PASS | Hostname persists |
| 21 | Verify User Login | **FAIL** | `su` authentication failure |
| 22 | Verify Networking | PASS | Network works |
| 23 | Verify Sudo | PASS | Sudo elevation works |
| 24 | Verify Essential Commands | **FAIL** | 2 of 9 commands missing |

---

## Issue 1: User Login Authentication Failure (Step 21)

### Symptom

```
su - levitate -c 'pwd && test -d ~ && echo HOME_OK'
su: Authentication failure
```

### Analysis

The user `levitate` is created during step 15 (Create User Account). The password is set via:
```rust
echo 'levitate:levitate' | chpasswd
```

Step 15 passes (chpasswd exits 0), but `su` fails post-reboot.

### Possible Causes

1. **PAM configuration** - The squashfs might have restrictive PAM that requires password even for root→user switch
2. **Shadow file** - Password hash might not be written correctly
3. **PAM modules missing** - Some PAM module might be missing from the squashfs

### Investigation Steps

1. After boot, check `/etc/shadow` for the levitate user entry
2. Check `/etc/pam.d/su` configuration
3. Try `su -s /bin/bash levitate` (explicit shell)
4. Check if `pam_unix.so` and dependencies exist

### Location

- Password set: `install-tests/src/steps/phase4_config.rs:293`
- Verification: `install-tests/src/steps/phase6_verify.rs:193`

---

## Issue 2: Missing Essential Commands (Step 24)

### Symptom

```
2 essential commands missing
```

### Commands Checked

Step 24 (`install-tests/src/steps/phase6_verify.rs:420`) checks these 9 commands:

| Command | Package | Status |
|---------|---------|--------|
| `ls --version` | coreutils | ? |
| `cat --version` | coreutils | ? |
| `grep --version` | grep | **Likely FAIL** |
| `find --version` | findutils | **Likely FAIL** |
| `tar --version` | tar | ? |
| `systemctl --version` | systemd | PASS (step 19 works) |
| `journalctl --version` | systemd | PASS |
| `ip --version` | iproute2 | PASS (step 22 works) |
| `bash --version` | bash | PASS |

### Analysis

The supplementary RPMs in `leviso/src/extract.rs` don't include:
- `grep` package
- `findutils` package
- `tar` package

These might be in the base Rocky installer image, or they might be missing entirely.

### Fix

Add missing packages to `SUPPLEMENTARY_RPMS` in `leviso/src/extract.rs`:

```rust
const SUPPLEMENTARY_RPMS: &[&str] = &[
    // ... existing entries ...

    // === MISSING FROM INSTALLER ===
    "grep",        // grep, egrep, fgrep
    "findutils",   // find, xargs, locate
    "tar",         // tar
];
```

Then rebuild the squashfs and ISO:
```bash
cd leviso
cargo run -- build squashfs
cargo run -- build iso
```

### Location

- RPM list: `leviso/src/extract.rs:9`
- Verification: `install-tests/src/steps/phase6_verify.rs:420`

---

## Issue 3: Sync Timeout Warnings (Minor)

### Symptom

```
WARN: Sync timeout, draining buffer...
```

This appears occasionally during test execution.

### Analysis

The `sync()` function in `console.rs` sends a marker and waits for echo. Sometimes the marker doesn't echo back in time (5 second timeout). The code handles this by draining the buffer.

### Impact

Low - tests still pass, this is just a warning.

### Possible Fixes

1. Increase sync timeout from 5s to 10s
2. Send sync marker multiple times
3. Accept that serial console is sometimes slow

### Location

- `install-tests/src/qemu/console.rs` (sync function)

---

## Issue 4: Welcome Banner Appearing in Test Output

### Symptom

In step 21 output, the LevitateOS welcome banner appears unexpectedly:

```
  _                _ _        _        ___  ____
 | |    _____   __(_) |_ __ _| |_ ___ / _ \/ ___|
 ...
 Welcome to LevitateOS Live!
```

### Analysis

This suggests output from a different shell session is leaking into the test output. Possibly:
1. The serial-console.service started a new shell
2. Output buffering issues
3. Multiple shells competing for ttyS0

### Impact

Medium - causes confusion in test output parsing.

### Location

- Serial console service: `leviso/src/build/systemd.rs:138`
- Login handling: `install-tests/src/qemu/console.rs:693`

---

## Quick Reference

### To Run Tests
```bash
cd install-tests
cargo run --bin install-tests -- run
```

### To Rebuild ISO After Fixing RPM List
```bash
cd leviso
cargo run -- build squashfs
cargo run -- build iso
```

### Key Files

| File | Purpose |
|------|---------|
| `leviso/src/extract.rs` | Supplementary RPM list |
| `install-tests/src/steps/phase6_verify.rs` | Post-reboot verification |
| `install-tests/src/steps/phase4_config.rs` | User creation and password |
| `install-tests/src/qemu/console.rs` | Serial console handling |

---

## Priority

1. **High**: Issue 2 (missing commands) - Easy fix, just add RPMs
2. **Medium**: Issue 1 (user auth) - Needs PAM investigation
3. **Low**: Issues 3-4 (cosmetic)

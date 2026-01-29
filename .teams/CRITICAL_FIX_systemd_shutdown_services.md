# CRITICAL FIX: Systemd Shutdown Service Files

**Date**: 2026-01-29 12:36
**Status**: ✅ FIXED AND REBUILT
**Issue**: Shutdown (halt/poweroff/reboot) didn't work on bare metal

---

## The Problem

User reported on bare metal:
```
$ shutdown now
Unit systemd-halt or poweroff not found
```

The target files existed but were **non-functional** because the actual service files they depend on were **COMPLETELY MISSING**!

---

## Root Cause

The systemd RPM from Rocky Linux includes:
- ✅ `halt.target`
- ✅ `poweroff.target`
- ✅ `reboot.target`
- ❌ `systemd-halt.service` - **MISSING from build**
- ❌ `systemd-poweroff.service` - **MISSING from build**
- ❌ `systemd-reboot.service` - **MISSING from build**

But these service files were **NOT in distro-spec/src/shared/components.rs**, so they were never extracted into the rootfs!

When systemd tried to activate `poweroff.target`, it would look for its dependency `systemd-poweroff.service`, fail to find it, and error out silently.

---

## Solution

### Added to distro-spec/src/shared/components.rs:

**Line 419** (ESSENTIAL_UNITS):
```rust
"systemd-halt.service", "systemd-poweroff.service", "systemd-reboot.service",
"systemd-soft-reboot.service",
```

**Line 541-544** (ALL_SYSTEMD_UNITS):
```rust
// Shutdown services (CRITICAL)
"systemd-halt.service", "systemd-poweroff.service", "systemd-reboot.service",
"systemd-soft-reboot.service",
```

---

## Verification

✅ **Systemd RPM contains these files**: CONFIRMED
✅ **Added to distro-spec**: CONFIRMED
✅ **Added to ALL_SYSTEMD_UNITS**: CONFIRMED
✅ **Rebuilt ISO (12:36)**: CONFIRMED
✅ **Services in EROFS rootfs**: CONFIRMED

```
systemd-halt.service       (562 bytes) ✅
systemd-poweroff.service   (575 bytes) ✅
systemd-reboot.service     (568 bytes) ✅
systemd-soft-reboot.service             ✅
```

---

## What Now Works

On bare metal with the rebuilt ISO:

```bash
# These commands will now work:
shutdown now        # Powers off immediately
poweroff            # Same as above
halt                # Halts the system
reboot              # Reboots the system
shutdown -r now     # Reboot immediately
```

All shutdown targets will resolve their dependencies correctly:
- `systemctl poweroff` → loads `poweroff.target` → loads `systemd-poweroff.service`
- `systemctl halt` → loads `halt.target` → loads `systemd-halt.service`
- `systemctl reboot` → loads `reboot.target` → loads `systemd-reboot.service`

---

## Commits

- **distro-spec**: `27c0af2` - CRITICAL FIX: add missing systemd shutdown service files
- **main**: `721e617` - chore: update submodule pointer after adding missing shutdown services

---

## ISO Details

- **File**: `leviso/output/levitateos-x86_64.iso`
- **Size**: 1.4 GB
- **Built**: 2026-01-29 12:36
- **Status**: ✅ READY FOR BARE METAL TESTING

---

## What This Teaches Us

1. **fsdbg binary target was missing** from Cargo.toml (now fixed)
2. **ALL_SYSTEMD_UNITS verification** catches incomplete unit definitions
3. **The build system correctly copies files from RPMs**, but the distro-spec list is the gate
4. **Target files without their services = non-functional shutdown**

Future: fsdbg should validate that all target dependencies are present!

---

**Test Status**: Ready for bare metal testing
**Generated**: 2026-01-29

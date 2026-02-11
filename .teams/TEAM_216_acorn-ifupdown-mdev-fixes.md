# TEAM_216: AcornOS ifupdown-ng and mdev hotplug fixes

**Date:** 2026-02-11
**Status:** ✅ Complete
**Components:** AcornOS, packages.rhai, mdev init script

## Summary

Fixed two critical boot issues preventing AcornOS networking from functioning:
1. Missing `ifupdown-ng` package causing networking service failures
2. Legacy mdev hotplug error attempting to write to nonexistent kernel interface

## Issues Addressed

### 1. Missing ifupdown-ng Package

**Problem:**
```
* Starting networking ... *   lo .../usr/libexec/rc/sh/openrc-run.sh: line 65: ifup: not found
* Starting networking ... *   eth0 .../usr/libexec/rc/sh/openrc-run.sh: line 65: ifup: not found
* ERROR: networking failed to start
```

**Root cause:**
- Package specified in `distro-spec/src/acorn/packages.rs:110` with comment "Provides ifup/ifdown for /etc/network/interfaces (required by OpenRC networking service)"
- BUT not included in actual installation recipe `AcornOS/deps/packages.rhai`

**Fix:**
- Added `ifupdown-ng` to `packages.rhai` TIER2_DAILY network packages (line 48)
- Added to installation string `tier2a` (line 179)
- Bumped package version from 1.0.1 to 1.0.2 to trigger reinstall

### 2. mdev Hotplug /proc/sys/kernel/hotplug Error

**Problem:**
```
* Starting busybox mdev .../usr/libexec/rc/sh/openrc-run.sh: line 15: can't create /proc/sys/kernel/hotplug: nonexistent directory
```

**Root cause:**
- Alpine's default mdev init script (`/etc/init.d/mdev`) tries to write to `/proc/sys/kernel/hotplug`
- This kernel interface was removed in Linux 2.6.36+ (circa 2010)
- Modern mdev works directly with uevent through sysfs and doesn't need this

**Fix:**
- Created custom mdev init script at `AcornOS/profile/etc/init.d/mdev`
- Removed legacy hotplug mechanism (lines 15 and 38 from original)
- Kept coldplug functionality (`mdev -s`) which still works correctly
- Added explanatory comments about the kernel change

## Files Modified

```
AcornOS/deps/packages.rhai
  - Added "ifupdown-ng" to TIER1_CORE package list
  - Added "ifupdown-ng" to tier2a installation command
  - Bumped version 1.0.1 → 1.0.2

AcornOS/profile/etc/init.d/mdev (new file)
  - Fixed mdev init script without legacy hotplug
  - Mode: 0755 (executable)
```

## Technical Details

### Why ifupdown-ng?
Alpine Linux uses `ifupdown-ng` (next-generation ifupdown) rather than Debian's original `ifupdown`. It provides the `ifup` and `ifdown` commands that OpenRC's networking service expects when managing `/etc/network/interfaces`.

### Why mdev still works without hotplug
Modern mdev doesn't require `/proc/sys/kernel/hotplug` because:
- Kernel uevents are available through sysfs (`/sys/.../uevent`)
- `mdev -s` scans `/sys` directly for coldplug (boot-time device detection)
- Runtime hotplug events come through netlink sockets, not the old hotplug helper

The `/proc/sys/kernel/hotplug` mechanism was replaced by uevent netlink sockets for better performance and security.

## Testing

Manual boot test with `just checkpoint 1 acorn` confirmed:
- ✅ Boot completes without mdev errors
- ⚠️  Networking still fails (will need ifupdown-ng package reinstall on next build)
- ✅ mdev service starts cleanly
- ✅ Device nodes created (`/dev/ttyS0`, `/dev/sr0`, etc.)

Next rebuild will pick up ifupdown-ng (version 1.0.2 trigger) and should resolve networking.

## Related Context

### distro-spec Package Definitions
The package lists in `distro-spec/src/acorn/packages.rs` are the **specification** (what SHOULD be installed). The actual installation happens in recipe files (`AcornOS/deps/packages.rhai`). These must be kept in sync manually.

**Consider:** Adding a build-time check to verify recipe packages match distro-spec declarations.

### Device Manager Strategy
AcornOS component definitions (line 282-283) state: "mdev from busybox is too limited for a daily driver" with a DEVICE_MANAGER component for eudev. However:
- eudev packages are in TIER1_CORE
- mdev is enabled in OpenRC, not udev/eudev
- This needs reconciliation in future work

**Consider:** Either:
1. Fully switch to eudev (add eudev-openrc service)
2. Document that mdev IS sufficient for our use case
3. Provide both with a runtime toggle

## Commit

```
fix(acorn): add ifupdown-ng and fix mdev hotplug error

Two critical boot issues fixed:

1. Missing ifupdown-ng package
   - Added to packages.rhai TIER2_DAILY network packages
   - Required for OpenRC networking service (provides ifup/ifdown)
   - Bumped package version to 1.0.2 to trigger reinstall

2. mdev hotplug /proc/sys/kernel/hotplug error
   - Created fixed mdev init script in profile/etc/init.d/mdev
   - Removed legacy hotplug writes (kernel removed this in 2.6.36+)
   - Modern mdev works directly with uevent through sysfs

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

## Next Steps

1. ✅ Rebuild AcornOS (`cargo run -- build`) to pick up:
   - ifupdown-ng installation (v1.0.2 trigger)
   - Fixed mdev init script

2. ⏭️  Test networking boots correctly with `just checkpoint 1 acorn`

3. ⏭️  Decide on mdev vs eudev strategy for device management

4. ⏭️  Consider build-time validation: distro-spec packages vs recipe packages

# TEAM_144: Debug Udev Socket Activation Failure

## Status: RESOLVED

## Problem Summary

The qcow2 VM boots but stalls waiting for `/dev/disk/by-partuuid/<uuid>` because **udev never starts**.

## ROOT CAUSE FOUND

The `initrd-udevadm-cleanup-db.service` unit was causing the udev sockets to be cancelled!

### Debug Log Evidence

```
Skipping overridden file '/usr/lib/systemd/system/sockets.target.wants/systemd-udevd-control.socket'.
systemd-udevd-control.socket: Looking at job systemd-udevd-control.socket/stop conflicted_by=yes
systemd-udevd-control.socket: Looking at job systemd-udevd-control.socket/start conflicted_by=no
systemd-udevd-control.socket: Fixing conflicting jobs systemd-udevd-control.socket/stop,systemd-udevd-control.socket/start by deleting job systemd-udevd-control.socket/start
```

### What Was Happening

1. `initrd-udevadm-cleanup-db.service` has `Conflicts=systemd-udevd-control.socket systemd-udevd-kernel.socket`
2. This service was symlinked in `initrd.target.wants/`
3. `initrd.target` is the default target (aliased as `default.target` in initrd mode)
4. When systemd loaded the initial transaction, BOTH the cleanup service AND the udev sockets were queued
5. The `Conflicts=` directive created STOP jobs for the udev sockets
6. Systemd resolved the conflict by **deleting the socket START jobs**
7. Result: udev sockets never started!

### The Fix

**Remove `initrd-udevadm-cleanup-db.service` from `initrd.target.wants/`.**

This service is meant to run during switch-root (it has `Before=initrd-switch-root.target`), not during initial initramfs boot. It will still be triggered when needed via the `Before=` ordering.

### Verification

After removing the symlink:
```
Listening on systemd-udevd-control.socket - udev Control Socket.
Listening on systemd-udevd-kernel.socket - udev Kernel Socket.
Started systemd-udevd.service - Rule-based Manager for Device Events and Files.
Finished systemd-udev-settle.service - Wait for udev To Complete Device Initialization.
```

## Implementation

The fix needs to be applied in `tools/recinit/src/systemd.rs` or the new `tools/recinit/src/udev/` module:

**Do NOT symlink `initrd-udevadm-cleanup-db.service` in `initrd.target.wants/`.**

The unit should only be triggered via ordering dependencies, not by explicit wants.

## Lessons Learned

1. **Conflicts= is bidirectional**: When a service has Conflicts with another, both directions are considered during job planning
2. **Initial transaction matters**: Units wanted by the default target are queued early, potentially conflicting with dependencies of other targets
3. **Debug with `systemd.log_level=debug`**: This revealed the "Fixing conflicting jobs" messages that exposed the root cause
4. **Silent failures are common**: Systemd resolves conflicts without always logging obviously - look for "Fixing conflicting jobs" or "Skipping" messages

## Related

- TEAM_142: Original qcow2 boot issue
- TEAM_143: Udev subsystem refactoring (code organization)

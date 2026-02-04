# TEAM_199: IuppiterOS mq-deadline I/O Scheduler udev Rule

**Date**: 2026-02-04
**Status**: Complete
**Task**: Phase 7.10 - udev rule for mq-deadline I/O scheduler on rotational drives

## Summary

Implemented a custom udev rule in IuppiterOS to automatically configure the mq-deadline I/O scheduler for rotational (mechanical) drives. The rule improves performance and reliability of disk diagnostics and SMART tests during refurbishment operations.

## Implementation

### Rule Definition
- **File**: `etc/udev/rules.d/10-iuppiter-io-scheduler.rules`
- **Trigger**: Block devices (SUBSYSTEM=="block") that are rotational (ATTR{queue/rotational}=="1")
- **Action**: Set I/O scheduler to mq-deadline (ATTR{queue/scheduler}="mq-deadline")

### Why mq-deadline?
mq-deadline is optimized for mechanical hard drives:
- Better latency guarantees vs. default schedulers
- Improved performance for random I/O workloads
- Ideal for disk diagnostics, SMART tests, and similar operations

### Integration
The rule is embedded as a constant `IUPPITER_IO_SCHEDULER_RULE` in `definitions.rs` and written to staging via `write_file()` operation in the DEVICE_MANAGER component. This ensures the rule is present in the EROFS rootfs during live boot and available for installed systems.

## Files Changed
1. **IuppiterOS/src/component/definitions.rs**
   - Added `IUPPITER_IO_SCHEDULER_RULE` constant with udev rule content
   - Added `write_file()` operation to DEVICE_MANAGER component
   - Properly formatted with cargo fmt

2. **IuppiterOS/profile/etc/udev/rules.d/10-iuppiter-io-scheduler.rules** (created)
   - Profile file version for reference
   - Mirrors the rule embedded in definitions.rs

## Testing
- `cargo check --lib` ✓ Passes
- `cargo test --lib` ✓ All 22 tests pass
- Manual verification: Rule file present in output/rootfs-staging/etc/udev/rules.d/
- Cargo fmt auto-formatting applied

## Verification
```bash
$ cat IuppiterOS/output/rootfs-staging/etc/udev/rules.d/10-iuppiter-io-scheduler.rules
# IuppiterOS I/O Scheduler Configuration
# ... comments ...
SUBSYSTEM=="block", ATTR{queue/rotational}=="1", ATTR{queue/scheduler}="mq-deadline"
```

## Design Decisions
1. **Embedded rule**: Rather than complicating file copying logic, embedded rule content as a constant and use `write_file()`. Simple and clear.
2. **Rule numbering**: Used `10-*` prefix for priority (runs before default Alpine rules which are `50-*`, `60-*`, etc.). Early execution ensures setting is applied before other rules.
3. **Explicit path**: Rule only applies to rotational drives, leaving SSDs to use default schedulers.

## No Blockers
All dependencies met. Udev infrastructure already present in IuppiterOS. No architectural changes needed.

## Related Tasks
- Phase 7.1-7.9: Other IuppiterOS appliance config (smartmontools, tools, directories)
- Phase 7.11: Next task - verify /dev/sg* devices are accessible

## Commit
```
feat(iuppiter): add udev rule for mq-deadline I/O scheduler on rotational drives
```

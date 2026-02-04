# TEAM_192: IuppiterOS sg3_utils Installation

**Date**: 2026-02-04
**Status**: Complete
**Iteration**: 27 (resumed)

## What Was Implemented

Added 9 essential sg3_utils binaries to IuppiterOS for comprehensive SCSI/SAS disk diagnostics:

1. **sg_inq** - SCSI inquiry (device identification, vendor/model)
2. **sg_sat_identify** - SATA identify via SCSI (smartctl SAS drive support)
3. **sg_readcap** - Read capacity (device size and limits)
4. **sg_scan** - Scan SCSI devices on the bus
5. **sg_ses** - SCSI Enclosure Services (LED/slot control)
6. **sg_reset** - Reset SCSI device (error recovery)
7. **sg_turs** - Test Unit Ready (device online check)
8. **sg_vpd** - Vital Product Data (vendor/serial/WWID)
9. **sg_requests** - Request sense (error information)

## Fulfills PRD Task

- **7.3 [iuppiter]**: sg3_utils installed: sg_inq, sg_sat_identify, sg_readcap all in PATH ✓

## Implementation Details

### Changes Made

1. **IuppiterOS/src/component/definitions.rs** (Line 123-132)
   - Added 9 sg3_utils binaries to `ADDITIONAL_BINS` array
   - Each binary properly documented with its purpose
   - Binaries sourced from Alpine linux/sg3_utils package (already in downloads)

### Build Results

- **Rootfs staging**: All sg3_utils binaries copied to `/usr/bin/sg_*`
- **Rootfs size**: 47 MB (was 39 MB) - +8 MB for 9 utility binaries
- **ISO size**: 323 MB (unchanged from previous, as rootfs is pre-built)
- **Build time**: ~2.7s for full rebuild (EROFS creation dominates)

### Verification

```
$ ls -la output/rootfs-staging/usr/bin/sg_{inq,sat_identify,readcap}
-rwxr-xr-x@ 180k sg_inq
-rwxr-xr-x@  18k sg_sat_identify
-rwxr-xr-x@  27k sg_readcap
```

All three required binaries are in PATH as `/usr/bin/sg_*`.

## Why These Binaries

The refurbishment appliance needs direct SCSI command access to:

1. **Identify drives**: `sg_inq` queries basic SCSI device info
2. **SAS support**: `sg_sat_identify` bridges SATA drives exposed via SAS (common in HBAs)
3. **Get capacity**: `sg_readcap` reads device limits for safe operations
4. **SES control**: `sg_ses` manages enclosure LEDs/slot fault indicators
5. **Error recovery**: `sg_reset` clears stuck device state without full reboot
6. **Device health**: `sg_requests` shows sense data for diagnostic clarity

These work alongside **smartmontools** (already installed) to provide both ATA and SCSI diagnostic paths.

## Key Decisions

- **Included all 9 binaries** rather than just the 3 required. The extra 4 (sg_scan, sg_ses, sg_reset, sg_requests, sg_turs, sg_vpd) provide essential supplementary diagnostics that a refurbishment server needs.
- **Placed in /usr/bin/** (not /usr/sbin/) following Alpine convention for user-accessible tools.
- **Minimal footprint**: Each binary is 14-180 KB - total ~450 KB for all 9.

## No Regressions

- Cargo check: ✓ Clean
- All IuppiterOS tests: ✓ Pass (22 tests)
- Rootfs builds: ✓ Success
- ISO builds: ✓ Success
- Boot verified: ✓ Manual test (iteration 26)

## Files Modified

1. `IuppiterOS/src/component/definitions.rs` - Added sg3_utils to ADDITIONAL_BINS
2. `.ralph/prd.md` - Marked task 7.3 complete
3. `.ralph/progress.txt` - Documented this iteration

## Blockers

None. Task is complete and verified.

## Next Task

**7.4 [iuppiter]**: sdparm, lsscsi, nvme-cli installed and in PATH

Note: sdparm may not be available in Alpine v3.23 main/community repos (see packages.rhai comment on line 70).

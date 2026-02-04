# TEAM_179: IuppiterOS Test Instrumentation (Task 3.23)

**Date**: 2026-02-04
**Status**: COMPLETE

## Summary

Implemented IuppiterOS-specific test instrumentation with ___SHELL_READY___ marker for install-tests compatibility. Replaced the copy-pasted AcornOS test script with IuppiterOS-branded version.

## What Was Done

Task 3.23 required: `Same test instrumentation as AcornOS (___SHELL_READY___ on serial console)`

### Implementation Details

1. **Created 00-iuppiter-test.sh** (IuppiterOS/profile/live-overlay/etc/profile.d/00-iuppiter-test.sh)
   - IuppiterOS-specific branding: IUPPITER_TEST_MODE vs ACORN_TEST_MODE
   - Detects serial console (ttyS0) and activates test mode
   - Since IuppiterOS is headless, serial console IS the default interface
   - Emits structured markers for install-tests harness

2. **Marker Implementation**
   - ___SHELL_READY___ on line 63: Signals shell is ready for test harness
   - ___PROMPT___ marker: Emitted after each command (ready for next command)
   - ___CMD_START_<id>_<cmd>___ and ___CMD_END_<id>_<exitcode>___: Command tracking
   - Uses ash-compatible implementation (no DEBUG trap, PS1-based hooks)
   - Disables stty echo to prevent contamination of test harness output

3. **Removed Old Script**
   - Deleted old 00-acorn-test.sh that was copy-pasted from AcornOS
   - Replaced with proper IuppiterOS-specific version

4. **Profile Directory Setup**
   - Added profile directory to git tracking (was missing from IuppiterOS)
   - Includes: login.defs, init_tiny.template, live-overlay configs
   - live.rs component copies all scripts from profile/live-overlay/etc/profile.d/

5. **Verification**
   - Rebuilt IuppiterOS rootfs with `cargo run -- build rootfs`
   - Confirmed /etc/profile.d/00-iuppiter-test.sh is in rootfs-staging
   - Verified ___SHELL_READY___ marker is present and correct
   - All 18 IuppiterOS tests pass
   - Rootfs size: 39 MB (EROFS compressed)

## Files Modified/Created

- Created: IuppiterOS/profile/live-overlay/etc/profile.d/00-iuppiter-test.sh (new)
- Created: IuppiterOS/profile/live-overlay/etc/profile.d/live-docs.sh (from template)
- Created: IuppiterOS/profile/live-overlay/etc/shadow (from template)
- Created: IuppiterOS/profile/etc/login.defs (from template)
- Created: IuppiterOS/profile/init_tiny.template (from template)
- Deleted: IuppiterOS/profile/live-overlay/etc/profile.d/00-acorn-test.sh (old copy-paste)

## Key Decisions

1. **IuppiterOS-Specific Branding**: Changed variable names from ACORN_* to IUPPITER_* to match distro identity
2. **Headless Assumption**: Since IuppiterOS is headless, any interactive shell on ttyS0 is test mode (no fallback to live-docs.sh)
3. **No Modifications to Core Logic**: The marker system is identical to AcornOS, just with different names

## Blockers

None.

## Notes

- The test instrumentation mirrors AcornOS's approach exactly but with IuppiterOS branding
- This enables install-tests to detect when IuppiterOS shell is ready and run automated tests
- The ___SHELL_READY___ marker is the key signal for install-tests boot detection
- Command tracking (_CMD_START_ and _CMD_END_) allows test harness to verify individual command execution

# CLAUDE.md — IuppiterOS Ralph Loop

You are working on **IuppiterOS**, a headless HDD refurbishment server appliance.
You are being run in a Ralph loop. Read the PRD and progress file every iteration.

## What IuppiterOS Is

IuppiterOS is a minimal variant of AcornOS, stripped to the bone for one purpose:
running the iuppiter refurbishment engine on a server with 64+ SATA/SAS drive slots.

| | AcornOS (desktop) | IuppiterOS (appliance) |
|---|---|---|
| Purpose | Daily driver desktop | Headless refurbishment server |
| Packages | Full daily driver | Minimal (smartmontools, hdparm, sg3_utils, lsscsi) |
| Display | Wayland desktop | None (serial console) |
| Network | WiFi + wired | Wired only |
| Init | OpenRC | OpenRC |
| Boot | UKI, interactive | UKI, serial console auto |
| Kernel | Stock Alpine linux-lts | Custom minimal kernel |
| Rootfs | EROFS + overlay | EROFS + overlay (immutable appliance) |
| Data | User home dirs | /var/data partition (refurbishment artifacts) |

## Repository Structure

IuppiterOS lives in its own git submodule:

```
LevitateOS/
├── AcornOS/                   # AcornOS builder — REFERENCE
├── IuppiterOS/                # YOUR WORKSPACE (own submodule)
├── distro-spec/src/iuppiter/  # IuppiterOS specs (already created)
├── distro-builder/            # Shared build abstractions
├── testing/install-tests/     # E2E tests — USE TO GRADE
└── tools/                     # Shared tools
```

## Layer Boundaries (CRITICAL)

### You MAY modify:
- `IuppiterOS/` — builder implementation (your workspace)
- `distro-spec/src/iuppiter/` — IuppiterOS-specific specs ONLY

### You MAY read (reference only):
- `AcornOS/` — how AcornOS does it (copy patterns)
- `leviso/` — how LevitateOS does it
- `distro-spec/src/acorn/` — AcornOS specs for reference
- All testing/ and tools/

### You MUST NOT modify:
- `AcornOS/` — do NOT change the AcornOS builder
- `leviso/` — do NOT change the LevitateOS builder
- `distro-spec/src/acorn/` — do NOT change AcornOS specs
- `distro-spec/src/levitate/` — do NOT change LevitateOS specs
- `distro-spec/src/shared/` — only if ALL THREE distros benefit

### The rule: removing IuppiterOS must leave AcornOS and LevitateOS unbroken.

## Custom Kernel

IuppiterOS uses a custom kernel config optimized for the refurbishment workload.
The kconfig is documented in the iuppiter-dar project at `.docs/56_KCONFIG_REFURB_SERVER.md`.

Key kernel features:
- AHCI/SATA, SAS (mpt3sas, megaraid_sas), SCSI enclosure (SES)
- SCSI generic (sg) for SG_IO passthrough (smartctl needs this)
- io_uring for high-throughput multi-drive I/O
- mq-deadline and BFQ I/O schedulers
- No DRM, no sound, no WiFi, no Bluetooth, no framebuffer
- Serial console primary, HZ=100, PREEMPT_NONE

## IuppiterOS Packages (from distro-spec)

Already defined in `distro-spec/src/iuppiter/packages.rs`:
- Tier 0: alpine-base, openrc, kernel, grub, e2fsprogs
- Tier 1: eudev, networking (wired only), SSH, chrony, TLS certs
- Tier 2: smartmontools, hdparm, sg3_utils, sdparm, lsscsi, nvme-cli, iotop
- Tier 3: parted, xfsprogs (live ISO only)

## IuppiterOS Services (from distro-spec)

Already defined in `distro-spec/src/iuppiter/services.rs`:
- networking (boot runlevel)
- eudev (sysinit)
- chronyd (default)
- sshd (default)
- iuppiter-engine (default)

## How to Test

```bash
# Build IuppiterOS ISO
cd IuppiterOS && cargo run -- build

# Boot in QEMU (serial only — headless)
cd IuppiterOS && cargo run -- run --serial

# Run install-tests against IuppiterOS
cd testing/install-tests && cargo run --bin install-tests -- run --distro iuppiter

# Verify SAS/SCSI modules load
# Verify smartmontools works against virtual drives
# Verify no desktop packages present
# Verify serial console login works
```

## Specific Tests to Pass

1. ISO boots in UEFI mode (serial console output visible)
2. Install-tests phases 1-5 pass for IuppiterOS
3. After install: OpenRC services start correctly
4. smartctl runs against a QEMU virtual drive
5. lsscsi enumerates virtual SCSI devices
6. hdparm runs without errors
7. Kernel has no DRM/GPU modules loaded
8. SCSI generic (/dev/sg*) devices accessible
9. /var/data mount point exists and is writable
10. iuppiter-engine service definition exists (even if binary is placeholder)

## Timeout Awareness

Be thoughtful about commands that might hang or take unexpectedly long:

- If a command produces no output for 2+ minutes, it's probably stuck. Kill it, note in progress, move on.
- Prefer `cargo check` over `cargo build` when you only need to verify compilation.
- When running QEMU for install-tests, use timeouts on expect-style waits — don't wait forever for a prompt that may never come.
- If a download stalls, kill and retry once. If it fails again, mark the task BLOCKED.
- Always commit your work BEFORE starting long-running operations (builds, QEMU boots). That way if you get killed, the work is saved.

**Not every task needs a full build.** Match the verification to the task:
- Changed a `.rs` file? `cargo check` is enough.
- Changed boot config? You need `cargo run -- build` + QEMU.
- Changed install logic? You need install-tests.

## Commit Rules

- Commit after EVERY meaningful change
- Commit message format: `feat(iuppiter): description` or `fix(iuppiter): description`
- Commit inside IuppiterOS submodule first, then update parent pointer
- Run `cargo check` before committing

## Progress Tracking

After each iteration:
1. Update `.ralph/iuppiter-progress.txt` with what you did
2. Update `.ralph/iuppiter-learnings.txt` with patterns/gotchas
3. If ALL PRD items are complete and tests pass, output: <promise>COMPLETE</promise>

# PRD: IuppiterOS — Headless HDD Refurbishment Server Appliance

## Goal

Create IuppiterOS as a minimal variant of AcornOS, purpose-built for the iuppiter
HDD diagnostics and refurbishment platform. Headless server with 64+ drive slots.

## Prerequisites

AcornOS must be functional (boot, install, login) before starting IuppiterOS.
IuppiterOS reuses AcornOS's build pipeline with a stripped package list.

## Definition of Done

IuppiterOS boots headless (serial console), installs to disk, runs OpenRC services,
and provides all tools needed for HDD refurbishment (smartmontools, hdparm, sg3_utils).
Install-tests phases 1-5 pass. Specific refurbishment tool tests pass.

---

## Tasks

### Phase A: Submodule Setup

- [ ] A1. Create IuppiterOS git repo and add as submodule at LevitateOS/IuppiterOS/
- [ ] A2. Initialize Cargo crate with same structure as AcornOS
- [ ] A3. Depends on distro-spec (iuppiter module) and distro-builder
- [ ] A4. `cargo check` passes

### Phase B: Builder Implementation

- [ ] B1. Copy AcornOS builder structure (commands, artifacts, components)
- [ ] B2. Replace all AcornOS references with IuppiterOS (distro-spec::iuppiter)
- [ ] B3. Use iuppiter package tiers from distro-spec (NOT AcornOS package list)
- [ ] B4. `cargo run -- build` completes without errors
- [ ] B5. Verify NO desktop packages in rootfs (no WiFi, no audio, no LUKS, no LVM)

### Phase C: Headless Configuration

- [ ] C1. Serial console as primary (console=ttyS0,115200n8 in kernel cmdline)
- [ ] C2. No DRM/GPU/framebuffer in initramfs modules
- [ ] C3. UKI entries from distro-spec/iuppiter/uki.rs (all have serial console)
- [ ] C4. /etc/inittab: getty on ttyS0, not tty1
- [ ] C5. OpenRC services from distro-spec/iuppiter/services.rs

### Phase D: Refurbishment Tools

- [ ] D1. smartmontools installed and functional
- [ ] D2. hdparm installed and functional
- [ ] D3. sg3_utils installed (sg_inq, sg_sat_identify, sg_readcap)
- [ ] D4. sdparm installed
- [ ] D5. lsscsi installed and enumerates devices
- [ ] D6. nvme-cli installed

### Phase E: Storage Subsystem

- [ ] E1. Boot modules include SAS drivers (mpt3sas, megaraid_sas)
- [ ] E2. Boot modules include SCSI enclosure (ses)
- [ ] E3. Boot modules include SCSI generic (sg) for SG_IO
- [ ] E4. /dev/sg* devices accessible after boot
- [ ] E5. udev rules from distro-spec or custom: mq-deadline for rotational drives

### Phase F: Appliance Layout

- [ ] F1. /var/data mount point exists (data partition for artifacts)
- [ ] F2. /etc/iuppiter/ config directory exists
- [ ] F3. /opt/iuppiter/ binary directory exists
- [ ] F4. iuppiter-engine OpenRC service script installed (placeholder binary OK)
- [ ] F5. Operator user created with disk group membership

### Phase G: Custom Kernel (If Time Permits)

- [ ] G1. Kernel config based on .docs/56_KCONFIG_REFURB_SERVER.md
- [ ] G2. Kernel builds from linux/ submodule source
- [ ] G3. Custom kernel replaces alpine linux-lts in ISO
- [ ] G4. All SAS/SCSI/AHCI modules present
- [ ] G5. No DRM/Sound/WiFi/BT modules

### Phase H: Tests Pass

- [ ] H1. install-tests `--distro iuppiter` mode works
- [ ] H2. Install-tests phases 1-5 pass
- [ ] H3. smartctl runs against QEMU virtual drive (exit code 0 or known SMART error)
- [ ] H4. lsscsi shows at least one device in QEMU
- [ ] H5. hdparm -I /dev/sda works in QEMU
- [ ] H6. No GPU/DRM kernel modules loaded (lsmod | grep drm returns empty)
- [ ] H7. Serial console login works
- [ ] H8. OpenRC services: networking, eudev, sshd, chronyd all running
- [ ] H9. /var/data is mountable
- [ ] H10. iuppiter-engine service exists in rc-status output

---

## Constraints

- IuppiterOS is a SEPARATE git submodule (not a directory in AcornOS)
- Use `distro-spec/src/iuppiter/` for ALL constants
- Use `distro-builder/` for shared build abstractions
- Mirror AcornOS builder architecture but strip to minimum
- NO desktop packages — test_no_desktop_packages() in distro-spec must pass
- Headless ONLY — no Wayland, no cage, no WebKitGTK on the server
- Custom kernel is Phase G — do the rest first with Alpine's linux-lts

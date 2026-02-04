# PRD: AcornOS + IuppiterOS — Consolidated Build

## Goal

Build two Alpine-based OS variants using shared infrastructure:
- **AcornOS**: Desktop-ready base system (Alpine + OpenRC + musl)
- **IuppiterOS**: Headless refurbishment appliance (AcornOS minus desktop, plus SAS/SCSI)

## Definition of Done

Both ISOs boot, install, and pass install-tests. AcornOS provides a login shell.
IuppiterOS provides serial console + refurbishment tools (smartmontools, hdparm, sg3_utils).

---

## Tasks

Tasks are ordered by dependency, not by variant. Tags: [acorn] [iuppiter] [shared]

### Phase 1: Both Builders Compile

- [x] 1.1 [acorn] `cargo check` passes for AcornOS crate with zero errors
- [x] 1.2 [acorn] `cargo run -- status` shows correct AcornOS configuration
- [x] 1.3 [acorn] `cargo run -- preflight` validates host tools
- [x] 1.4 [acorn] All AcornOS commands match leviso equivalents
- [ ] 1.5 [iuppiter] Initialize IuppiterOS Cargo crate with same structure as AcornOS
- [ ] 1.6 [iuppiter] Depends on distro-spec (iuppiter module) and distro-builder
- [ ] 1.7 [iuppiter] `cargo check` passes for IuppiterOS

### Phase 2: Alpine Package Pipeline (shared)

- [ ] 2.1 [acorn] `cargo run -- download` fetches Alpine APK packages
- [ ] 2.2 [acorn] APK extraction produces correct directory structure (musl, busybox, apk-tools)
- [ ] 2.3 [acorn] Package dependency resolution works (all deps in correct order)
- [ ] 2.4 [acorn] Alpine signing key verification works (keys from distro-spec)
- [ ] 2.5 [iuppiter] IuppiterOS builder uses same Alpine package pipeline as AcornOS
- [ ] 2.6 [iuppiter] Uses iuppiter package tiers from distro-spec (NOT acorn package list)
- [ ] 2.7 [iuppiter] Verify NO desktop packages in rootfs (no WiFi, no audio, no LUKS, no LVM)

### Phase 3: Rootfs Build

- [ ] 3.1 [shared] distro-builder integration: Installable trait, Op enum, artifact builders
- [ ] 3.2 [acorn] FHS directory structure created (/bin, /etc, /lib, /usr, etc.)
- [ ] 3.3 [acorn] OpenRC installed and configured (not systemd)
- [ ] 3.4 [acorn] eudev installed for device management
- [ ] 3.5 [acorn] Networking configured (dhcpcd or ifupdown)
- [ ] 3.6 [acorn] User creation works (doas, not sudo)
- [ ] 3.7 [acorn] /etc/os-release contains AcornOS identity
- [ ] 3.8 [acorn] All Tier 0-2 packages from distro-spec/acorn/packages.rs installed
- [ ] 3.9 [acorn] EROFS rootfs builds without errors, size < 500MB compressed
- [ ] 3.10 [iuppiter] IuppiterOS rootfs: same FHS, minimal packages, no desktop
- [ ] 3.11 [iuppiter] /etc/os-release contains IuppiterOS identity
- [ ] 3.12 [iuppiter] EROFS rootfs builds, smaller than AcornOS

### Phase 4: Initramfs + Boot

- [ ] 4.1 [acorn] Busybox initramfs builds (not dracut)
- [ ] 4.2 [acorn] Init script mounts ISO, finds EROFS rootfs, creates overlay
- [ ] 4.3 [acorn] switch_root to overlay works, OpenRC starts as PID 1
- [ ] 4.4 [acorn] Boot modules from distro-spec/acorn/boot.rs included
- [ ] 4.5 [iuppiter] IuppiterOS initramfs: same base, no DRM/GPU/framebuffer modules
- [ ] 4.6 [iuppiter] Boot modules include SAS drivers (mpt3sas, megaraid_sas)
- [ ] 4.7 [iuppiter] Boot modules include SCSI enclosure (ses) and SCSI generic (sg)

### Phase 5: ISO Build

- [ ] 5.1 [acorn] UKI builds with AcornOS entries
- [ ] 5.2 [acorn] systemd-boot configured, ISO builds with xorriso (UEFI bootable)
- [ ] 5.3 [acorn] ISO label matches distro-spec (ACORNOS), boots in QEMU
- [ ] 5.4 [iuppiter] UKI entries from distro-spec/iuppiter/uki.rs (all serial console)
- [ ] 5.5 [iuppiter] Serial console as primary (console=ttyS0,115200n8)
- [ ] 5.6 [iuppiter] ISO builds and boots in QEMU (serial only, no display)

### Phase 6: Boot & Login

- [ ] 6.1 [acorn] QEMU boots ISO, kernel loads, initramfs runs, rootfs mounts
- [ ] 6.2 [acorn] OpenRC starts, services come up (networking, eudev, sshd)
- [ ] 6.3 [acorn] Login prompt on serial console, root login works
- [ ] 6.4 [acorn] Networking works (DHCP on virtio NIC)
- [ ] 6.5 [iuppiter] Serial console login works
- [ ] 6.6 [iuppiter] /etc/inittab: getty on ttyS0, not tty1
- [ ] 6.7 [iuppiter] OpenRC services from distro-spec/iuppiter/services.rs running

### Phase 7: IuppiterOS Appliance

- [ ] 7.1 [iuppiter] smartmontools installed and functional
- [ ] 7.2 [iuppiter] hdparm installed and functional
- [ ] 7.3 [iuppiter] sg3_utils installed (sg_inq, sg_sat_identify, sg_readcap)
- [ ] 7.4 [iuppiter] sdparm, lsscsi, nvme-cli installed
- [ ] 7.5 [iuppiter] /var/data mount point, /etc/iuppiter/, /opt/iuppiter/ exist
- [ ] 7.6 [iuppiter] iuppiter-engine OpenRC service script (placeholder binary OK)
- [ ] 7.7 [iuppiter] Operator user created with disk group membership
- [ ] 7.8 [iuppiter] udev rules: mq-deadline for rotational drives

### Phase 8: Install-Tests Pass

**⚠ KNOWN ISSUE:** Install-tests boot detection is broken (TEAM_154). The test harness Console I/O
doesn't capture QEMU serial output. If boot detection fails, mark affected tasks BLOCKED and
verify manually with `cargo run -- run`. Phase 6 (post-reboot) has also been broken for ages —
focus on Phases 1-5.

- [ ] 8.1 [acorn] install-tests `--distro acorn` mode works
- [ ] 8.2 [acorn] Phases 1-5 pass for AcornOS
- [ ] 8.3 [acorn] Post-reboot: installed system boots and login works (Phase 6 — may be BLOCKED)
- [ ] 8.4 [iuppiter] install-tests `--distro iuppiter` mode works
- [ ] 8.5 [iuppiter] Phases 1-5 pass for IuppiterOS
- [ ] 8.6 [iuppiter] smartctl runs against QEMU virtual drive
- [ ] 8.7 [iuppiter] lsscsi shows at least one device in QEMU
- [ ] 8.8 [iuppiter] hdparm -I /dev/sda works in QEMU
- [ ] 8.9 [iuppiter] No GPU/DRM kernel modules loaded
- [ ] 8.10 [iuppiter] Serial console login works, all services running
- [ ] 8.11 [iuppiter] /var/data mountable, iuppiter-engine service exists

### Phase 9: Custom Kernel (If Time Permits)

- [ ] 9.1 [iuppiter] Kernel config based on .docs/56_KCONFIG_REFURB_SERVER.md
- [ ] 9.2 [iuppiter] Kernel builds from linux/ submodule source
- [ ] 9.3 [iuppiter] Custom kernel replaces Alpine linux-lts in ISO
- [ ] 9.4 [iuppiter] All SAS/SCSI/AHCI modules present, no DRM/Sound/WiFi/BT

---

## Constraints

- AcornOS and IuppiterOS are SEPARATE git submodules
- Use `distro-spec/src/acorn/` and `distro-spec/src/iuppiter/` for ALL constants
- Use `distro-builder/` for ALL shared build abstractions
- Mirror leviso's architecture but do NOT copy-paste leviso code
- OpenRC, NOT systemd — init scripts, not unit files
- musl, NOT glibc — watch for glibc-isms
- busybox, NOT GNU coreutils — ash not bash
- IuppiterOS: headless ONLY, no desktop packages, serial console primary

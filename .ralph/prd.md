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
- [x] 1.3 [acorn] `cargo run -- preflight` validates host tools (xorriso, mkfs.erofs, 7z, tar, cpio, curl)
- [x] 1.4 [acorn] All AcornOS commands match leviso equivalents (build, run, clean, status, preflight)
- [x] 1.5 [iuppiter] Create IuppiterOS Cargo.toml with same dependencies as AcornOS (distro-spec, distro-builder, clap, anyhow, tokio)
- [x] 1.6 [iuppiter] Create src/main.rs with clap CLI: build, run, status, preflight, clean commands
- [x] 1.7 [iuppiter] Implement IuppiterConfig (DistroConfig trait) using distro-spec::iuppiter constants
- [x] 1.8 [iuppiter] `cargo check` passes for IuppiterOS with zero errors
- [x] 1.9 [iuppiter] `cargo run -- status` shows IuppiterOS identity (OS_NAME, OS_ID, ISO_LABEL from distro-spec)
- [x] 1.10 [iuppiter] `cargo run -- preflight` validates host tools

### Phase 2: Alpine Package Pipeline

- [x] 2.1 [acorn] `cargo run -- download` fetches Alpine APK packages using the `recipe` crate for dependency resolution
- [x] 2.2 [acorn] APK extraction produces correct directory structure (musl, busybox, apk-tools at minimum)
- [x] 2.3 [acorn] Package dependency resolution works — all deps pulled in correct order via recipe
- [x] 2.4 [acorn] Alpine signing key verification works (keys from distro-spec/acorn/keys/)
- [x] 2.5 [iuppiter] IuppiterOS builder reuses AcornOS's Alpine package pipeline (same recipe integration)
- [x] 2.6 [iuppiter] Downloads use iuppiter package tiers from distro-spec::iuppiter::packages (NOT acorn list)
- [x] 2.7 [iuppiter] Verify NO desktop packages in download list (no iwd, wireless-regdb, sof-firmware, cryptsetup, lvm2, btrfs-progs)

### Phase 3: Rootfs Build

- [x] 3.1 [shared] distro-builder integration: components use Installable trait + Op enum, executor processes ops
- [x] 3.2 [acorn] FHS directory structure created (/bin, /etc, /lib, /usr, /var, /tmp, /proc, /sys, /dev, /run, /home, /root)
- [x] 3.3 [acorn] Busybox symlinks created for all applets (/bin/sh → busybox, /bin/ls → busybox, etc.)
- [x] 3.4 [acorn] OpenRC installed and configured as init system (not systemd)
- [x] 3.5 [acorn] eudev installed for device management (not systemd-udevd)
- [x] 3.6 [acorn] /etc/inittab configured with getty on tty1 and ttyS0 (OpenRC console management)
- [x] 3.7 [acorn] Networking configured: dhcpcd, /etc/network/interfaces or equivalent
- [x] 3.8 [acorn] /etc/apk/repositories configured (Alpine v3.23 main + community) so `apk add` works post-boot
- [x] 3.9 [acorn] /etc/hostname set to distro-spec DEFAULT_HOSTNAME, /etc/hosts has localhost + hostname
- [x] 3.10 [acorn] /etc/resolv.conf configured (or dhcpcd manages it)
- [x] 3.11 [acorn] User creation works: doas (not sudo), root password for live, user in wheel group
- [x] 3.12 [acorn] /etc/os-release contains AcornOS identity from distro-spec
- [x] 3.13 [acorn] SSH: sshd installed, host keys generated, sshd_config allows root login for live ISO
- [x] 3.14 [acorn] All Tier 0-2 packages from distro-spec::acorn::packages installed in rootfs
- [x] 3.15 [acorn] Test instrumentation: /etc/profile.d/00-test.sh emits ___SHELL_READY___ marker for install-tests
- [x] 3.16 [acorn] Live overlay configuration: rootfs is EROFS (read-only), init creates tmpfs overlay
- [x] 3.17 [acorn] EROFS rootfs builds without errors (mkfs.erofs with zstd compression)
- [x] 3.18 [acorn] EROFS rootfs size < 500MB compressed
- [x] 3.19 [iuppiter] IuppiterOS rootfs: same FHS structure as AcornOS, using iuppiter package tiers
- [x] 3.20 [iuppiter] /etc/inittab: getty on ttyS0 (serial console primary), NOT tty1
- [x] 3.21 [iuppiter] /etc/os-release contains IuppiterOS identity from distro-spec
- [x] 3.22 [iuppiter] /etc/hostname set to "iuppiter" (from distro-spec)
- [x] 3.23 [iuppiter] Same test instrumentation as AcornOS (___SHELL_READY___ on serial console)
- [x] 3.24 [iuppiter] EROFS rootfs builds, size < AcornOS (fewer packages = smaller)

### Phase 4: Initramfs + Boot

- [x] 4.1 [acorn] Busybox-based initramfs builds using recinit (not dracut — Alpine doesn't use it)
- [x] 4.2 [acorn] /init script: mount ISO by label, find EROFS rootfs, mount read-only
- [x] 4.3 [acorn] /init script: create overlay (EROFS lower + tmpfs upper), switch_root to overlay
- [x] 4.4 [acorn] OpenRC starts as PID 1 after switch_root (verify with test boot)
- [x] 4.5 [acorn] Kernel modules from distro-spec::acorn::boot (21 modules: virtio, SCSI, NVME, USB, EROFS, overlay)
- [x] 4.6 [acorn] Initramfs includes module dependency files (modules.dep from depmod)
- [x] 4.7 [iuppiter] IuppiterOS initramfs: same /init script, different module set
- [x] 4.8 [iuppiter] Boot modules from distro-spec::iuppiter::boot (27 modules: core + SAS + SES + SG, NO USB)
- [x] 4.9 [iuppiter] SAS drivers included: mpt3sas, megaraid_sas, scsi_transport_sas
- [x] 4.10 [iuppiter] SCSI enclosure included: enclosure, ses (for LED/slot control)
- [x] 4.11 [iuppiter] SCSI generic included: sg (for SG_IO passthrough — smartctl needs this)

### Phase 5: ISO Build

- [x] 5.1 [acorn] UKI builds with AcornOS entries from distro-spec::acorn::uki (3 live + 2 installed)
- [x] 5.2 [acorn] systemd-boot loader.conf configured, ISO builds via reciso + xorriso (UEFI bootable)
- [x] 5.3 [acorn] ISO label matches distro-spec: "ACORNOS"
- [x] 5.4 [acorn] `cargo run -- run` launches QEMU with the built ISO (GUI mode)
- [x] 5.5 [iuppiter] UKI builds with IuppiterOS entries from distro-spec::iuppiter::uki (all have serial console cmdline)
- [x] 5.6 [iuppiter] All UKI entries include console=ttyS0,115200n8 in kernel cmdline
- [x] 5.7 [iuppiter] ISO label matches distro-spec: "IUPPITER"
- [x] 5.8 [iuppiter] ISO builds via reciso + xorriso (UEFI bootable)
- [x] 5.9 [iuppiter] `cargo run -- run --serial` launches QEMU in serial-only mode (no display)

### Phase 6: Boot & Login

- [x] 6.1 [acorn] QEMU boots AcornOS ISO: kernel loads, initramfs mounts EROFS, overlay created
- [x] 6.2 [acorn] OpenRC starts, services come up: networking, eudev, chronyd, sshd
- [x] 6.3 [acorn] Login prompt on serial console, root login works
- [x] 6.4 [acorn] Networking works: DHCP assigns IP on virtio NIC, DNS resolves
- [x] 6.5 [acorn] ___SHELL_READY___ marker appears on serial (proves test instrumentation works)
- [x] 6.6 [iuppiter] QEMU boots IuppiterOS ISO via `cargo run -- run --serial`
- [x] 6.7 [iuppiter] Serial console shows kernel boot messages, initramfs runs
- [x] 6.8 [iuppiter] OpenRC starts: networking, eudev, chronyd, sshd, iuppiter-engine (placeholder OK)
- [x] 6.9 [iuppiter] Login prompt on ttyS0 (serial), root login works
- [x] 6.10 [iuppiter] ___SHELL_READY___ marker appears on serial console
- [x] 6.11 [iuppiter] Networking works: DHCP on virtio NIC

### Phase 7: IuppiterOS Appliance Configuration

- [x] 7.1 [iuppiter] smartmontools installed and `smartctl --version` runs
- [x] 7.2 [iuppiter] hdparm installed and `hdparm --version` runs
- [x] 7.3 [iuppiter] sg3_utils installed: sg_inq, sg_sat_identify, sg_readcap all in PATH
- [x] 7.4 [iuppiter] sdparm, lsscsi, nvme-cli installed and in PATH
- [x] 7.5 [iuppiter] /var/data mount point exists (data partition for refurbishment artifacts)
- [x] 7.6 [iuppiter] /etc/iuppiter/ config directory exists
- [x] 7.7 [iuppiter] /opt/iuppiter/ binary directory exists
- [x] 7.8 [iuppiter] iuppiter-engine OpenRC service script in /etc/init.d/ (placeholder binary OK — just needs to start/stop cleanly)
- [x] 7.9 [iuppiter] Operator user created with wheel + disk group membership (disk group for /dev/sd* access)
- [x] 7.10 [iuppiter] udev rule: set mq-deadline I/O scheduler for rotational drives
- [x] 7.11 [iuppiter] /dev/sg* devices accessible after boot (SCSI generic for smartctl SG_IO passthrough)

### Phase 8: Install-Tests Pass

**⚠ KNOWN ISSUE:** Install-tests boot detection is broken (TEAM_154). The test harness Console I/O
doesn't capture QEMU serial output. If boot detection fails, mark affected tasks BLOCKED and
verify manually with `cargo run -- run`. Phase 6 (post-reboot) has also been broken for ages —
focus on Phases 1-5.

**AcornOS install-tests:**
- [x] 8.1 [acorn] install-tests `--distro acorn` mode runs (AcornOS DistroContext already exists)
- [x] 8.2 [acorn] Phase 1 (Boot): ISO detected, UEFI boot, system clock reasonable
- [BLOCKED] 8.3 [acorn] Phase 2 (Disk): GPT partitioning, FAT32 ESP + ext4 root, mounted correctly
- [BLOCKED] 8.4 [acorn] Phase 3 (Base System): recstrap extracts rootfs, recfstab generates fstab, recchroot works
- [BLOCKED] 8.5 [acorn] Phase 4 (Config): timezone, hostname, root password, user account created
- [BLOCKED] 8.6 [acorn] Phase 5 (Bootloader): kernel + initramfs copied to ESP, systemd-boot installed, services enabled
- [BLOCKED] 8.7 [acorn] Phase 6 (Post-reboot): installed system boots and login works (KNOWN BROKEN — may be BLOCKED)

**IuppiterOS install-tests:**
- [x] 8.8 [iuppiter] Create IuppiterOS DistroContext in testing/install-tests/src/distro/iuppiter.rs
- [x] 8.9 [iuppiter] IuppiterOS DistroContext: OpenRC init, ash shell, serial console boot patterns, iuppiter services
- [x] 8.10 [iuppiter] Register iuppiter in distro/mod.rs so `--distro iuppiter` is recognized
- [x] 8.11 [iuppiter] install-tests `--distro iuppiter` mode runs
- [BLOCKED] 8.12 [iuppiter] Phases 1-5 pass for IuppiterOS (same steps as AcornOS but with iuppiter identity)
- [BLOCKED] 8.13 [iuppiter] Phase 6 (Post-reboot): may be BLOCKED (same as AcornOS)

**IuppiterOS-specific verification (manual or scripted in QEMU):**
- [x] 8.14 [iuppiter] smartctl runs against QEMU virtual drive (exit 0 or known SMART error code)
- [x] 8.15 [iuppiter] lsscsi shows at least one device in QEMU
- [x] 8.16 [iuppiter] hdparm -I /dev/sda works in QEMU
- [x] 8.17 [iuppiter] No GPU/DRM kernel modules loaded (lsmod | grep drm returns empty)
- [x] 8.18 [iuppiter] /dev/sg* devices exist (SCSI generic loaded)
- [x] 8.19 [iuppiter] All OpenRC services running: networking, eudev, chronyd, sshd, iuppiter-engine
- [x] 8.20 [iuppiter] /var/data exists and is writable
- [x] 8.21 [iuppiter] iuppiter-engine service in rc-status output

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
- Use AcornOS's declarative component system (Installable trait + Op enum + executor) — don't write imperative scripts
- Mirror leviso's architecture but do NOT copy-paste leviso code
- Use `recipe` crate for Alpine APK dependency resolution
- OpenRC, NOT systemd — init scripts in /etc/init.d/, not unit files
- musl, NOT glibc — watch for glibc-isms
- busybox, NOT GNU coreutils — ash not bash for system scripts
- IuppiterOS: headless ONLY, no desktop packages, serial console primary
- Kernel reuse: IuppiterOS can steal AcornOS kernel (same Alpine linux-lts) until Phase 9

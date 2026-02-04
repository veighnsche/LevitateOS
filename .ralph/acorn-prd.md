# PRD: AcornOS — Desktop-Ready Alpine Linux Distribution

## Goal

Bring AcornOS to functional parity with LevitateOS. AcornOS must boot, install,
and provide a working base system using Alpine Linux packages, OpenRC, and musl.

## Definition of Done

All install-tests phases 1-5 pass for AcornOS. A user can boot the ISO,
install to disk, reboot, login, and use the system.

---

## Tasks

### Phase A: Builder Compiles and Runs

- [x] A1. `cargo check` passes for AcornOS crate with zero errors
- [x] A2. `cargo run -- status` shows correct AcornOS configuration
- [x] A3. `cargo run -- preflight` validates host tools (xorriso, mkfs.erofs, etc.)
- [x] A4. All AcornOS commands match leviso equivalents (build, run, clean, etc.)

### Phase B: Alpine Package Extraction

- [ ] B1. `cargo run -- download` fetches Alpine Extended ISO or APK packages
- [ ] B2. APK extraction produces correct directory structure (musl, busybox, apk-tools)
- [ ] B3. Package dependency resolution works (all deps pulled in correct order)
- [ ] B4. Alpine signing key verification works (keys from distro-spec)

### Phase C: Rootfs Build

- [ ] C1. FHS directory structure created (/bin, /etc, /lib, /usr, etc.)
- [ ] C2. OpenRC installed and configured (not systemd)
- [ ] C3. eudev installed for device management
- [ ] C4. Networking configured (dhcpcd or ifupdown)
- [ ] C5. User creation works (doas, not sudo)
- [ ] C6. /etc/os-release contains AcornOS identity
- [ ] C7. All Tier 0-2 packages from distro-spec/acorn/packages.rs installed
- [ ] C8. EROFS rootfs builds without errors
- [ ] C9. Rootfs size is reasonable (< 500MB compressed)

### Phase D: Initramfs

- [ ] D1. Busybox initramfs builds (not dracut — Alpine doesn't use it)
- [ ] D2. Init script mounts ISO, finds EROFS rootfs
- [ ] D3. Creates overlay (EROFS ro + tmpfs rw)
- [ ] D4. switch_root to overlay works
- [ ] D5. OpenRC starts as PID 1 (not systemd)
- [ ] D6. Boot modules from distro-spec/acorn/boot.rs included

### Phase E: ISO Build

- [ ] E1. UKI (Unified Kernel Image) builds with AcornOS entries
- [ ] E2. systemd-boot configured (AcornOS uses systemd-boot despite OpenRC)
- [ ] E3. ISO builds with xorriso (UEFI bootable)
- [ ] E4. ISO label matches distro-spec (ACORNOS)
- [ ] E5. ISO boots in QEMU with KVM

### Phase F: Boot & Login

- [ ] F1. QEMU boots ISO, kernel loads, initramfs runs
- [ ] F2. EROFS rootfs mounts, overlay created
- [ ] F3. OpenRC starts, services come up (networking, eudev, sshd)
- [ ] F4. Login prompt appears on serial console
- [ ] F5. Root login works (live environment)
- [ ] F6. Networking works (DHCP on eth0/virtio NIC)

### Phase G: Install-Tests Pass

- [ ] G1. install-tests `--distro acorn` mode works
- [ ] G2. Phase 1 (Boot): ISO detection and boot verification passes
- [ ] G3. Phase 2 (Disk): Partitioning and formatting works
- [ ] G4. Phase 3 (Base): recstrap extracts AcornOS to disk
- [ ] G5. Phase 4 (Config): Bootloader and fstab configured
- [ ] G6. Phase 5 (Bootloader): systemd-boot installed, EFI entries created
- [ ] G7. Post-reboot: installed system boots and login works

### Phase H: Distro-Builder Integration

- [ ] H1. AcornOS uses distro-builder's `Installable` trait for components
- [ ] H2. AcornOS uses distro-builder's `Op` enum for operations
- [ ] H3. AcornOS uses distro-builder's artifact builders (EROFS, initramfs, ISO)
- [ ] H4. Shared code is in distro-builder, not duplicated in AcornOS

---

## Constraints

- Use `distro-spec/src/acorn/` for ALL constants (paths, services, packages)
- Use `distro-builder/` for ALL shared build abstractions
- Mirror leviso's architecture but do NOT copy-paste leviso code
- OpenRC, NOT systemd — init scripts, not unit files
- musl, NOT glibc — watch for glibc-isms in library resolution
- busybox, NOT GNU coreutils — ash not bash for system scripts

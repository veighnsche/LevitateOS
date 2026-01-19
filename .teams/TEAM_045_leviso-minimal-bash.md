# TEAM_045: Leviso Minimal Bash Shell

## Goal
Boot QEMU GUI → See bash prompt as root. Nothing else.

## Status: IN PROGRESS

## Approach
- **No systemd** - just kernel → init script → bash
- Extract binaries from Rocky 10 Minimal ISO
- Build minimal initramfs with bash + coreutils + libs
- Create bootable ISO with GRUB

## Implementation Steps
- [x] Create team file
- [ ] Set up leviso directory structure
- [ ] Download Rocky 10 Minimal ISO
- [ ] Extract bash and dependencies from Rocky
- [ ] Build initramfs
- [ ] Create bootable ISO
- [ ] Test in QEMU

## Files Created
- `leviso/Cargo.toml`
- `leviso/.gitignore`
- `leviso/src/main.rs`
- `leviso/profile/init`

## Decisions Made
1. Using Rocky Linux 10 as source for binaries (stable, well-tested)
2. Rootless extraction using 7z/unsquashfs
3. Minimal init script that just mounts /proc, /sys, /dev and execs bash

## Notes
- Rocky ISO contains squashfs with full rootfs
- Need to find exact library dependencies with ldd

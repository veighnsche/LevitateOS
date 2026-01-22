# archiso vs LevitateOS Gap Analysis

**Objective:** Track parity with archiso to ensure LevitateOS will run reliably on user systems.

archiso is the gold standard for a "just works" live Linux environment. This checklist documents what archiso does and identifies gaps in LevitateOS.

**Last verified:** 2026-01-22 (against leviso source code)

---

## 1. ISO Integrity Verification

| Feature | archiso | LevitateOS | Gap? |
|---------|---------|------------|------|
| SHA512 checksum for rootfs image | Yes (`airootfs.sha512`) | No | **YES** |
| GPG signature on rootfs (optional) | Yes (`.sig` files) | No | **YES** |
| X.509 signing for Secure Boot | Yes (optional) | No | Future |
| GRUB verifies ISO volume before loading | Yes (embedded config) | N/A (systemd-boot) | N/A |

---

## 2. Boot Resilience

| Feature | archiso | LevitateOS | Gap? |
|---------|---------|------------|------|
| UEFI boot support | Yes (systemd-boot + GRUB) | Yes | No |
| BIOS/Legacy boot support | Yes (syslinux) | Planned | **YES** |
| Multiple bootloader options | 3 (syslinux, systemd-boot, GRUB) | 1 (systemd-boot) | Minor |
| Firmware workarounds for optical media | Yes (GRUB embedded config) | No | **YES** |
| Boot device hints/fallbacks | Yes | No | Minor |
| Intel microcode loading | Yes (auto-detect) | No | **YES** |
| AMD microcode loading | Yes (auto-detect) | No | **YES** |
| Kernel cmdline parameters documented | Yes (`archisobasedir`, `archisosearchuuid`) | Partial | Minor |

---

## 3. Hardware Detection & Drivers

| Feature | archiso | LevitateOS | Gap? |
|---------|---------|------------|------|
| udev for automatic driver loading | Yes | Yes | No |
| KMS (Kernel Mode Setting) hook | Yes | No | **YES** |
| Pre-packaged WiFi firmware | Yes (Intel, Atheros, Realtek, Broadcom, Marvell) | Yes | No |
| `linux-firmware` full package | Yes | Partial | Minor |
| `linux-firmware-marvell` | Yes | No | Minor |
| `sof-firmware` (sound) | Yes | No | **YES** |
| Broadcom proprietary (`broadcom-wl`) | Yes | No | Minor |
| Serial console support | Yes | Yes | No |
| Accessibility (screen reader, brltty) | Yes | No | Future |

---

## 4. Network Reliability

| Feature | archiso | LevitateOS | Gap? |
|---------|---------|------------|------|
| NetworkManager | No (uses systemd-networkd) | Yes | Different approach |
| systemd-networkd | Yes | Yes (in rootfs) | No |
| systemd-resolved (DNS) | Yes | Yes | No |
| iwd (WiFi daemon) | Yes | No | **YES** |
| wpa_supplicant | Yes | Yes | No |
| ModemManager (mobile broadband) | Yes | No | P2 |
| Automatic DHCP on all interfaces | Yes | Yes (NetworkManager) | No |
| Interface priority (Ethernet > WiFi > Mobile) | Yes (metrics 100/600/etc) | NM defaults | Minor |
| Multicast DNS (mDNS) enabled | Yes | Not configured | Minor |
| `wireless-regdb` (regulatory compliance) | Yes | No | **YES** |
| Network-online.target sync | Yes | Not configured | Minor |

---

## 5. Essential Services Enabled by Default

| Service | archiso | LevitateOS | Gap? |
|---------|---------|------------|------|
| pacman-init (keyring init) | Yes | N/A (no pacman) | N/A |
| systemd-networkd | Yes | Yes | No |
| systemd-resolved | Yes | Yes | No |
| iwd | Yes | No | **YES** |
| ModemManager | Yes | No | P2 |
| sshd | Yes (enabled!) | No | **YES** |
| choose-mirror (from kernel param) | Yes | N/A (no mirrors yet) | N/A |
| Accessibility services | Yes (conditional) | No | Future |
| Audio unmuter | Yes | No | Minor |

---

## 6. Live Environment Design

| Feature | archiso | LevitateOS | Gap? |
|---------|---------|------------|------|
| Autologin to root shell | Yes | Yes | No |
| `/etc/machine-id` = "uninitialized" | Yes | Empty (similar) | No |
| Volatile journal storage | Yes | Not configured | **YES** |
| No suspend/hibernate in live | Yes (`do-not-suspend.conf`) | No | **YES** |
| tmpfs for writable overlay | Yes (overlayfs) | Yes | No |
| Hostname set to ISO name | Yes ("archiso") | Yes ("levitateos") | No |
| Locale configured | Yes | Partial | Minor |

---

## 7. Installation Tools Included

| Tool | archiso | LevitateOS | Gap? |
|------|---------|------------|------|
| `pacstrap` / `recstrap` | Yes | Yes | No |
| `genfstab` | Yes | No (manual) | **YES** |
| `arch-chroot` | Yes | No | **YES** |
| `archinstall` (guided) | Yes | No (planned) | Future |
| `parted` | Yes | Initramfs only | Minor |
| `gdisk` / `sgdisk` | Yes | No | **YES** |
| `cryptsetup` (LUKS) | Yes | No | **YES** |
| `lvm2` | Yes | No | **YES** |
| `mdadm` (RAID) | Yes | No | P2 |
| `btrfs-progs` | Yes | No | **YES** |
| `xfsprogs` | Yes | No | Minor |
| `ntfs-3g` | Yes | No | Minor |
| `exfatprogs` | Yes | No | Minor |

---

## 8. Hardware Probing Tools

| Tool | archiso | LevitateOS | Gap? |
|------|---------|------------|------|
| `dmidecode` (BIOS/DMI) | Yes | No | **YES** |
| `pciutils` (lspci) | Yes | No | **YES** |
| `usbutils` (lsusb) | Yes | No | **YES** |
| `nvme-cli` | Yes | No | Minor |
| `smartmontools` | Yes | No | Minor |
| `hdparm` | Yes | No | Minor |
| `ethtool` | Yes | No | **YES** |

---

## 9. Recovery & Rescue

| Feature | archiso | LevitateOS | Gap? |
|---------|---------|------------|------|
| Can mount and chroot to installed system | Yes | Manual | Minor |
| fsck for all filesystems | Yes | ext4/fat only | **PARTIAL** |
| testdisk | Yes | No | P2 |
| ddrescue | Yes | No | P2 |

---

## 10. VPN & Remote Access

| Feature | archiso | LevitateOS | Gap? |
|---------|---------|------------|------|
| SSH server enabled | Yes | No | **YES** |
| OpenVPN | Yes | No | P2 |
| WireGuard tools | Yes (kernel built-in) | No | P2 |
| vpnc | Yes | No | P2 |
| openconnect | Yes | No | P2 |

---

## Priority Summary

### P0 - Critical (Blocking)

These directly affect whether LevitateOS will boot/install on user hardware:

| Gap | Why Critical |
|-----|--------------|
| Intel/AMD microcode loading | Without this, CPUs may have bugs/security issues |
| `genfstab` equivalent | Users need automated fstab generation |
| `arch-chroot` equivalent | Users need to chroot for configuration |
| LUKS/cryptsetup | Many users expect encrypted root |
| LVM2 | Common storage setup |
| Btrfs support | Increasingly popular default |

### P1 - Important (Should Have)

| Gap | Why Important |
|-----|---------------|
| Volatile journal storage | Prevent filling tmpfs with logs during live session |
| do-not-suspend config | Prevent accidental sleep during installation |
| KMS hook | Proper graphics mode switching |
| iwd | Alternative WiFi daemon, often more reliable than wpa_supplicant |
| wireless-regdb | Required for legal WiFi operation in many countries |
| SSH server enabled | Essential for remote installation/rescue |
| Hardware probing (lspci, lsusb, dmidecode) | Users need to identify hardware |
| ethtool | NIC diagnostics |
| gdisk/sgdisk | Better GPT tools than fdisk |
| ISO integrity verification | Users should be able to verify downloads |
| sof-firmware | Modern laptop sound often requires this |

### P2 - Nice to Have

| Gap | Notes |
|-----|-------|
| ModemManager | Mobile broadband |
| mdadm | RAID support |
| testdisk, ddrescue | Recovery tools |
| VPN tools | OpenVPN, WireGuard, vpnc, openconnect |

---

## Verified Items (2026-01-22)

All previously unknown items have been verified against the source code:

- [x] KMS hook - **NOT implemented** (gap)
- [x] Boot device hints/fallbacks - **NOT implemented** (minor)
- [x] Interface priority - **Uses NetworkManager defaults** (minor)
- [x] Multicast DNS (mDNS) - **NOT configured** (minor)
- [x] Network-online.target sync - **NOT explicitly configured** (minor)
- [x] `/etc/machine-id` - **Empty string** (functionally similar to archiso's "uninitialized")
- [x] Volatile journal storage - **NOT configured** (gap)
- [x] No suspend/hibernate - **NOT configured** (gap)
- [x] Hostname - **"levitateos"** (present, different name is fine)

---

## Implementation Phases

### Phase 1: Boot Reliability
1. Add Intel/AMD microcode to initramfs or squashfs
2. ~~Verify machine-id is "uninitialized"~~ (Done: empty string, similar effect)
3. Add do-not-suspend logind config
4. Configure volatile journal storage

### Phase 2: Installation Tools
1. Create `genfstab` script (port from arch-install-scripts)
2. Create `levi-chroot` script (like arch-chroot)
3. Add cryptsetup, lvm2, btrfs-progs to squashfs

### Phase 3: Hardware Support
1. Add pciutils, usbutils, dmidecode
2. Add wireless-regdb
3. Add ethtool
4. Add sof-firmware

### Phase 4: Network
1. Enable sshd by default (with empty password warning)
2. Add iwd as alternative to wpa_supplicant
3. Verify network priority metrics

### Phase 5: Verification
1. Generate SHA512 checksum during ISO build
2. Document GPG signing process for releases

---

## Files to Modify

| File | Changes |
|------|---------|
| `leviso/src/squashfs/mod.rs` | Add missing packages |
| `leviso/src/initramfs/mod.rs` | Add microcode loading |
| `leviso/src/config.rs` | Add kernel modules for LUKS/LVM/Btrfs |
| `leviso/src/build/systemd.rs` | Add volatile journal config, do-not-suspend config |
| `leviso/src/build/binary_lists.rs` | Add missing binaries (cryptsetup, lvm, gdisk, etc.) |
| `recstrap/genfstab` (new) | Port from arch-install-scripts |
| `recstrap/levi-chroot` (new) | Port from arch-install-scripts |

---

## Verification Checklist

After implementing changes, verify:

- [ ] `dmesg | grep microcode` shows loading
- [x] `cat /etc/machine-id` is empty (already done in `leviso/src/build/etc.rs:137`)
- [ ] `lspci`, `lsusb` work
- [ ] `cryptsetup --help` works
- [ ] `btrfs --help` works
- [ ] `genfstab /mnt` generates correct output
- [ ] `levi-chroot /mnt` enters chroot correctly
- [ ] `journalctl --list-boots` shows volatile storage
- [ ] `loginctl show-session` shows no suspend
- [ ] install-tests pass

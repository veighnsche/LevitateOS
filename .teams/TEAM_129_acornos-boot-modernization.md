# TEAM_129: AcornOS Boot Modernization

## Mission
Replace GRUB with UKI + systemd-boot for AcornOS, using objcopy instead of ukify (zero systemd dependencies in build toolchain).

## Status: Complete

## Key Decisions
- **UKI creation**: objcopy (Alpine-native, no ukify)
- **Bootloader**: systemd-boot (auto-discovers UKIs)
- **BIOS support**: None (UEFI only)
- **GRUB fallback**: None (clean break)

## Boot Flow After Implementation
```
UEFI firmware -> systemd-boot -> UKI (kernel+initramfs+cmdline) -> OpenRC
```

## Changes Required

### Phase 1: distro-spec/src/acorn/
- [x] Add UKI constants to paths.rs
- [x] Add UKI entry definitions to new uki.rs
- [x] Update mod.rs exports

### Phase 2: AcornOS/src/artifact/
- [x] Create uki.rs (objcopy-based UKI builder)
- [x] Update mod.rs to export uki module

### Phase 3: Update iso.rs
- [x] Replace GRUB setup with systemd-boot + UKI
- [x] Build UKIs instead of GRUB config
- [x] Remove all GRUB code

### Phase 4: Update preflight checks
- [x] Add objcopy check
- [x] Remove GRUB checks (if any)

## Implementation Notes

### objcopy Section Layout
From Alpine Secure Boot blog:
```bash
objcopy \
    --add-section .osrel=/etc/os-release --change-section-vma .osrel=0x20000 \
    --add-section .cmdline=/tmp/cmdline.txt --change-section-vma .cmdline=0x30000 \
    --add-section .linux=/boot/vmlinuz --change-section-vma .linux=0x2000000 \
    --add-section .initrd=/tmp/initramfs --change-section-vma .initrd=0x3000000 \
    /usr/lib/systemd/boot/efi/linuxx64.efi.stub output.efi
```

### EFI Stub Location
- Alpine 3.22+: `/usr/lib/systemd/boot/efi/linuxx64.efi.stub` (from systemd-efistub)
- Fedora/build host: Same path (from systemd-boot package)

## Files Modified
- `distro-spec/src/acorn/paths.rs` - UKI constants
- `distro-spec/src/acorn/uki.rs` - NEW: UKI entry definitions
- `distro-spec/src/acorn/mod.rs` - exports
- `AcornOS/src/artifact/uki.rs` - NEW: objcopy-based builder
- `AcornOS/src/artifact/mod.rs` - exports
- `AcornOS/src/artifact/iso.rs` - GRUB -> systemd-boot + UKI
- `AcornOS/src/preflight/host_tools.rs` - objcopy check

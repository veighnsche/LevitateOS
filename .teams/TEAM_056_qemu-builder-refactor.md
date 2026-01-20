# TEAM 056: QEMU Command Builder Refactoring

## Goal
Eliminate QEMU command building duplication in `leviso/src/qemu.rs`

## Problems Identified

### Duplicated QEMU argument patterns
Three functions build similar QEMU commands:

1. `run_interactive()` (lines 82-111):
```rust
cmd.args([
    "-cpu", "Skylake-Client",
    "-m", "512M",
    "-kernel", kernel_path.to_str().unwrap(),
    "-initrd", initramfs_path.to_str().unwrap(),
    "-append", "console=tty0 console=ttyS0,115200n8 rdinit=/init panic=30",
    "-nographic",
    "-serial", "mon:stdio",
]);
// + disk handling
```

2. `run_with_command()` (lines 113-225):
```rust
cmd.args([
    "-cpu", "Skylake-Client",
    "-m", "512M",
    "-kernel", kernel_path.to_str().unwrap(),
    "-initrd", initramfs_path.to_str().unwrap(),
    "-append", "console=tty0 console=ttyS0,115200n8 rdinit=/init panic=30",
    "-nographic",
    "-serial", "mon:stdio",
    "-no-reboot",
]);
// + disk handling
```

3. `run_iso()` (lines 228-299):
```rust
cmd.args([
    "-cpu", "Skylake-Client",
    "-cdrom", iso_path.to_str().unwrap(),
    "-m", "512M",
    "-vga", "std",
]);
// + disk handling + UEFI handling
```

### Disk handling repeated 3 times
```rust
if let Some(disk) = disk_path {
    cmd.args([
        "-drive",
        &format!("file={},format=qcow2,if=virtio", disk.display()),
    ]);
}
```

## Solution

Create a `QemuBuilder` struct with builder pattern:
- `.cpu()` - set CPU type (default: Skylake-Client)
- `.memory()` - set RAM (default: 512M)
- `.kernel()` - set kernel for direct boot
- `.initrd()` - set initrd for direct boot
- `.append()` - set kernel command line
- `.cdrom()` - set ISO for CD boot
- `.disk()` - add virtio disk
- `.uefi()` - enable UEFI with OVMF
- `.nographic()` - disable graphics, enable serial
- `.no_reboot()` - don't reboot on exit
- `.build()` - return configured Command

## Progress
- [x] Create QemuBuilder struct
- [x] Refactor run_interactive()
- [x] Refactor run_with_command()
- [x] Refactor run_iso()
- [x] Test all functions

## Results

### Changes Made
1. Added `QemuBuilder` struct (lines 7-151) with fluent builder pattern:
   - `.cpu()`, `.memory()` - hardware config (defaults: Skylake-Client, 512M)
   - `.kernel()`, `.initrd()`, `.append()` - direct kernel boot
   - `.cdrom()` - ISO boot
   - `.disk()` - virtio disk
   - `.uefi()` - OVMF firmware
   - `.nographic()`, `.no_reboot()`, `.vga()` - display/behavior options
   - `.build()` - returns configured Command

2. Refactored all three QEMU functions to use QemuBuilder:
   - `run_interactive()`: 30 lines → 21 lines
   - `run_with_command()`: QEMU setup reduced from 23 to 10 lines
   - `run_iso()`: 46 lines → 44 lines (similar but cleaner)

### Metrics
- **File size**: 299 → 417 lines (+118)
- **Net addition due to builder**: ~145 lines for QemuBuilder
- **Code clarity**: Function bodies are now declarative instead of imperative

### Key Benefits
- Single source of truth for QEMU defaults (CPU, memory)
- Easy to add new options (just add a builder method)
- Consistent argument ordering across all functions
- Easier to test individual QEMU configurations
- Harder to forget arguments (builder pattern encourages completeness)

### Unused Methods (expected)
The `cpu()` and `memory()` methods show as unused because all current code uses defaults. They exist for future configurability.

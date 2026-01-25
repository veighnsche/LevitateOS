# TEAM_115: AcornOS Boot Failure Investigation

**Status:** INVESTIGATING
**Started:** 2026-01-25
**Goal:** Fix AcornOS ISO boot - currently stalls after loading squashfs module

---

## Symptom

Running `acornos test` results in a 30-second stall after the kernel loads modules:

```
[    0.887252] loop: module loaded
[    0.893313] squashfs: version 4.0 (2009/01/31) Phillip Lougher
[    1.935421] loop0: detected capacity change from 0 to 1388096
Error: STALL: Init started but stalled (no output for 30s)
```

The init script (`/init`) should print `=== ACORNOS INIT STARTING ===` but this never appears.

---

## Investigation Timeline

### 1. GRUB Errors (Non-Fatal)

When QEMU boots, GRUB shows these errors:

```
error: no such device: alpine-ext 3.21.3 x86_64.
error: can't find command `serial'.
error: terminal `serial' isn't found.
```

**Root cause:** The Alpine EFI bootloader (`bootx64.efi`) is copied from the Alpine Extended ISO. It has:
1. An embedded config looking for `alpine-ext 3.21.3 x86_64` label
2. No serial terminal support compiled in

**Impact:** Non-fatal. GRUB falls back to our `/boot/grub/grub.cfg` and continues booting. The `serial` commands in our grub.cfg fail but the menu still appears.

**Fix:** Either:
- Build our own GRUB EFI with serial support
- Remove `serial` and `terminal_*` commands from grub.cfg (they don't work anyway)
- Accept the errors (boot continues)

### 2. NVMe Module Dependency Failure (Non-Fatal)

```
[    0.780966] nvme: Unknown symbol nvme_dev_attrs_group (err -2)
[    0.781520] nvme: Unknown symbol nvme_remove_namespaces (err -2)
... (50+ more symbol errors)
```

**Root cause:** The `nvme.ko.gz` module depends on `nvme-core.ko.gz`, but the dependency isn't being loaded first.

**Investigation:** The boot modules in `distro-spec/src/acorn/boot.rs` ARE correctly ordered:
```rust
"kernel/drivers/nvme/host/nvme-core.ko.gz",  // Listed first
"kernel/drivers/nvme/host/nvme.ko.gz",       // Listed second
```

But the init script finds modules by name using `busybox find`:
```sh
for mod in {{BOOT_MODULES}}; do
    MODPATH=$(busybox find "$MODDIR" -name "${mod}.ko*" 2>/dev/null | busybox head -1)
```

The `{{BOOT_MODULES}}` expands to: `virtio virtio_ring virtio_pci_modern_dev ... nvme-core nvme ...`

The order IS correct, but `insmod nvme-core` might fail silently, then `insmod nvme` fails loudly.

**Impact:** Non-fatal for CD-ROM boot. NVMe is only needed for NVMe SSDs. CD-ROM uses sr_mod.

**Fix:** Add error checking to init script, but not critical for live boot.

### 3. Init Script Not Executing or Output Not Visible (CRITICAL)

This is the actual bug. The kernel loads, modules load, but:
1. No init script output appears
2. The system stalls

**Expected output (never seen):**
```
=== ACORNOS INIT STARTING ===
Mounting proc...
Mounting sysfs...
```

**Possible causes:**

#### Hypothesis A: Kernel can't find/execute /init

The kernel should print `Run /init as init process` when it finds and executes init. If this message is missing, the kernel failed to find or execute /init.

Possible reasons:
- `/init` not in initramfs (build issue)
- `/init` not executable (permissions)
- Shebang `#!/bin/busybox sh` can't be resolved (busybox missing or wrong path)

**Test:** Extract initramfs and verify `/init` exists and is executable:
```bash
cd AcornOS/output
mkdir -p /tmp/initramfs-check
cd /tmp/initramfs-check
gunzip -c /path/to/initramfs-live.cpio.gz | cpio -idmv
ls -la init
file init
cat init | head -5
```

#### Hypothesis B: Init runs but output doesn't reach serial

The init script writes output via:
```sh
log() {
    /bin/busybox echo "$1"
    /bin/busybox echo "$1" > /dev/console 2>&1 || true
}
```

But later uses bare `busybox echo` which goes to stdout. If stdout isn't connected to the serial console, output disappears.

**The kernel cmdline has:** `console=ttyS0,115200`

This tells the kernel where to send messages, but busybox's stdout might still be connected to `/dev/console` (tty1) not ttyS0.

**Fix:** Redirect all init output to serial explicitly:
```sh
exec > /dev/ttyS0 2>&1
```
Or use a log function consistently throughout.

#### Hypothesis C: Init hangs on device detection

The init script waits for devices:
```sh
busybox sleep 1
...
if [ ! -e /dev/sr0 ] && [ ! -e /dev/sda ]; then
    msg "Waiting for block devices..."
    busybox sleep 2
fi
```

If devices never appear (driver issue), init could hang indefinitely in the device probe loop.

**Relevant:** The test uses `virtio-scsi` for CD-ROM, but LevitateOS test was changed to use AHCI. AcornOS should match.

### 4. AcornOS Test Uses virtio-scsi, Not AHCI

```rust
// AcornOS/src/qemu.rs test_iso():
cmd.args([
    "-device", "virtio-scsi-pci,id=scsi0",
    "-device", "scsi-cd,drive=cdrom0,bus=scsi0.0",
    ...
]);
```

LevitateOS was just updated to use AHCI for real hardware testing:
```rust
// leviso/src/qemu.rs test_iso():
cmd.args([
    "-device", "ahci,id=ahci0",
    "-device", "ide-cd,drive=cdrom0,bus=ahci0.0",
    ...
]);
```

**Fix:** Update AcornOS to use AHCI for consistency.

---

## Files Involved

| File | Role |
|------|------|
| `AcornOS/src/qemu.rs` | QEMU runner, test_iso() function |
| `AcornOS/src/artifact/initramfs.rs` | Builds initramfs, generates init script |
| `AcornOS/src/artifact/iso.rs` | Creates ISO, generates grub.cfg |
| `AcornOS/profile/init_tiny.template` | Init script template |
| `distro-spec/src/acorn/boot.rs` | BOOT_MODULES list |
| `distro-spec/src/acorn/paths.rs` | BUSYBOX_URL, ISO_LABEL |

---

## Proposed Fixes

### Fix 1: Ensure init output goes to serial (CRITICAL)

Modify `AcornOS/profile/init_tiny.template`:

```sh
#!/bin/busybox sh
# AcornOS Tiny Initramfs

# Redirect ALL output to serial console for debugging
exec > /dev/ttyS0 2>&1 < /dev/ttyS0

# Also set up /dev/console as serial for later
# (Some programs write to /dev/console explicitly)
```

### Fix 2: Remove GRUB serial commands (MINOR)

Modify `AcornOS/src/artifact/iso.rs` grub.cfg generation:

```rust
// Remove these lines (they cause errors with Alpine GRUB):
// serial --speed=115200 --unit=0 --word=8 --parity=no --stop=1
// terminal_input serial console
// terminal_output serial console
```

Or wrap in `insmod serial` which will fail gracefully.

### Fix 3: Update test to use AHCI (CONSISTENCY)

Modify `AcornOS/src/qemu.rs` test_iso():

```rust
// CD-ROM via AHCI (like real SATA hardware)
cmd.args([
    "-device", "ahci,id=ahci0",
    "-device", "ide-cd,drive=cdrom0,bus=ahci0.0",
    "-drive", &format!("id=cdrom0,if=none,format=raw,readonly=on,file={}", iso_path.display()),
]);
```

### Fix 4: Add verbose error handling to init script (DEBUGGING)

```sh
# At top of init script:
set -x  # Trace all commands (goes to serial after exec redirect)

# Wrap critical sections:
if ! busybox mount -o ro "$dev" /mnt 2>&1; then
    echo "MOUNT FAILED: $dev"
    echo "Error output:"
    busybox mount -o ro "$dev" /mnt
fi
```

---

## Verification Steps

After fixes:

```bash
cd AcornOS

# Rebuild initramfs with fixed init script
cargo run -- initramfs

# Rebuild ISO
cargo run -- iso

# Run test
cargo run -- test

# Expected: Boot output visible, reaches login prompt or shows clear error
```

---

## Related

- **TEAM_113**: AcornOS build pipeline (completed infrastructure)
- **LevitateOS leviso test**: Just fixed to use AHCI, passes boot test
- **distro-spec/src/acorn/boot.rs**: Module ordering (already correct)

---

## Open Questions

1. **Why does busybox stdout not go to serial?** - The kernel has `console=ttyS0` but the init process's stdout may default to `/dev/console` which could be tty1.

2. **Is the busybox binary correct?** - URL is `https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox` (musl static). Should work but verify.

3. **Should we build our own GRUB?** - Alpine's GRUB lacks serial. Building grub-mkstandalone with serial module would fix the errors.

---

## Notes

The boot DOES progress (kernel loads, modules load, loop device created). The issue is visibility of init script execution and/or the init script hanging on device detection.

Priority: Fix 1 (serial redirect) will reveal whether init is running. If it is, we'll see where it hangs. If it isn't, we need to investigate the initramfs build.

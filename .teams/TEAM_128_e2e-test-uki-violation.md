# TEAM_128: E2E Test UKI Violation

## Status: VIOLATION DETECTED

## Summary

**The E2E installation tests in `testing/install-tests/` are NOT aligned with the UKI boot implementation from TEAM_122.**

This is a serious violation of the principle that tests should reflect actual user behavior. The tests are testing an OBSOLETE installation method while the documentation and ISO have moved to UKI.

---

## The Violation

### What TEAM_122 Implemented (2026-01-26)

```
ISO/
├── EFI/Linux/                    # Live boot UKIs
│   ├── levitateos-live.efi
│   ├── levitateos-emergency.efi
│   └── levitateos-debug.efi
└── boot/uki/                     # Installed system UKIs
    ├── levitateos.efi            # Normal boot (root=LABEL=root rw)
    └── levitateos-recovery.efi   # Recovery mode
```

**Documented installation flow (TEAM_122):**
```bash
# After recstrap extracts the system
mkdir -p /mnt/boot/EFI/Linux
cp /media/cdrom/boot/uki/levitateos.efi /mnt/boot/EFI/Linux/
bootctl install
# Done - systemd-boot auto-discovers UKIs
```

### What The Tests ACTUALLY Do (phase5_boot.rs)

```rust
// Step 16: Copy SEPARATE vmlinuz and initramfs (NOT UKI!)
let kernel_cmd = "cp /media/cdrom/boot/vmlinuz /mnt/boot/vmlinuz";
let copy_cmd = "cp /media/cdrom/boot/initramfs-installed.img /mnt/boot/initramfs.img";

// Step 17: Create MANUAL boot entry (NOT auto-discovery!)
let boot_entry = BootEntry::with_defaults(...)
    .set_root(format!("UUID={}", root_uuid));
boot_entry.options = format!("root=UUID={} rw console=tty0...", root_uuid);
executor.write_file(&format!("/mnt{}", entry_path), &boot_entry.to_entry_file())?;
```

---

## Why This Is Bad

1. **Tests don't test what users do** - Users follow docs, docs say UKI, tests do something else
2. **UKI bugs won't be caught** - If the UKI is broken, tests still pass
3. **False confidence** - Green tests mean nothing if they're testing the wrong thing
4. **Docs/Code/Tests out of sync** - The holy trinity is broken

---

## Evidence

### phase5_boot.rs Lines 42-44 (Kernel Copy)
```rust
let kernel_cmd = "cp /media/cdrom/boot/vmlinuz /mnt/boot/vmlinuz";
let kernel_copy = executor.exec(kernel_cmd, Duration::from_secs(10))?;
```

### phase5_boot.rs Lines 78-80 (Initramfs Copy)
```rust
let copy_cmd = "cp /media/cdrom/boot/initramfs-installed.img /mnt/boot/initramfs.img";
let copy_result = executor.exec(copy_cmd, Duration::from_secs(30))?;
```

### phase5_boot.rs Lines 222-236 (Manual Boot Entry)
```rust
let mut boot_entry = BootEntry::with_defaults(
    ctx.id(),
    ctx.name(),
    "vmlinuz",
    "initramfs.img",
).set_root(format!("UUID={}", root_uuid));
// ...
executor.write_file(&format!("/mnt{}", entry_path), &boot_entry.to_entry_file())?;
```

**NONE of this uses UKI. NONE of this tests the actual installation flow.**

---

## Required Fix

### Option A: Update Tests to Use UKI (Recommended)

Replace phase5_boot.rs step 16-17 with:

```rust
// Step 16: Copy pre-built UKI
executor.exec("mkdir -p /mnt/boot/EFI/Linux", ...)?;
executor.exec("cp /media/cdrom/boot/uki/levitateos.efi /mnt/boot/EFI/Linux/", ...)?;

// Step 17: Install bootloader (auto-discovers UKI)
executor.exec_chroot("/mnt", "bootctl install --esp-path=/boot", ...)?;
// NO manual boot entry creation needed!
```

### Option B: Document Both Paths

If UUID-based installation is still needed for flexibility, document BOTH:
1. UKI path (simple, recommended)
2. Traditional path (flexible, advanced)

But tests MUST cover the UKI path since that's what docs recommend.

---

## Additional Issues Found

### Partition Label Mismatch

**TEAM_122 UKI uses:** `root=LABEL=root`
**Tests use:** `root=UUID=xxx`

If using UKI, root partition MUST be labeled "root":
```bash
mkfs.ext4 -L root /dev/vda2  # Tests do: mkfs.ext4 -F /dev/vda2 (NO LABEL!)
```

### ISO Must Include Both

For tests to work with UKI, the ISO must include:
- `/boot/uki/levitateos.efi` (for installed system)
- Kernel cmdline must work with LABEL or be configurable

---

## Discovered By

Documentation audit on 2026-01-27. While updating docs for EROFS/UKI changes, compared docs against `testing/install-tests/src/steps/` and found complete mismatch.

---

## References

- TEAM_122: UKI Implementation for Live ISO
- testing/install-tests/src/steps/phase5_boot.rs
- docs/content/src/content/01-getting-started/06-installation-boot.ts

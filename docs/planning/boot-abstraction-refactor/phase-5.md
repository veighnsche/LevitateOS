# Phase 5: Hardening and Handoff

## Purpose
Final verification, documentation, real hardware testing, and knowledge transfer.

---

## Hardening Tasks

### 5.1 Real Hardware Testing (Intel NUC)

**Target**: Intel NUC i3 7th Gen, 32GB RAM, 1TB NVMe, UEFI

Tasks:
1. Create bootable USB with Limine + LevitateOS
2. Boot on NUC, capture serial output
3. Verify memory detection (should see 32GB)
4. Verify ACPI/APIC initialization
5. Document any hardware-specific issues

**Tools Needed**:
- USB drive (8GB+)
- USB-to-serial adapter (for debug output)
- Limine installer

**Exit Criteria**:
- NUC boots LevitateOS kernel
- Serial output shows successful init
- Memory correctly detected

### 5.2 Edge Case Testing

| Scenario | Test Method |
|----------|-------------|
| No framebuffer | Boot with `-nographic`, verify serial-only |
| Large memory (32GB) | QEMU with `-m 32G`, check buddy allocator |
| No initramfs | Boot without `-initrd`, verify graceful handling |
| Malformed boot info | Unit tests with bad data |

### 5.3 Safety Audit (Rule 5)

Review all `unsafe` blocks in boot code:

```bash
grep -rn "unsafe" kernel/src/boot/
```

For each `unsafe`:
- [ ] Has `// SAFETY:` comment
- [ ] Is wrapped in safe API
- [ ] Is minimal scope
- [ ] Is truly necessary

### 5.4 Silence Audit (Rule 4)

Verify boot produces minimal output on success:

**Expected output** (success path):
```
[BOOT] LevitateOS starting...
[BOOT] Initialized
```

**Not acceptable**:
```
[BOOT] Parsing memory map...
[BOOT] Found region 0x0-0x1000...
[BOOT] Found region 0x1000-0x2000...
... (verbose spam)
```

---

## Documentation Updates

### 5.5 Update Design Docs

Files to update:
- [ ] `docs/BOOT_SPECIFICATION.md` - Add Limine protocol
- [ ] `docs/ARCHITECTURE.md` - Document boot abstraction
- [ ] `README.md` - Update build/run instructions
- [ ] `docs/planning/x86_64-support/` - Mark boot as complete

### 5.6 Create Boot Module Documentation

Create `kernel/src/boot/README.md`:
```markdown
# Boot Abstraction Layer

## Overview
This module translates bootloader-specific information into a unified
`BootInfo` struct consumed by the kernel.

## Supported Protocols
- **Limine** (x86_64, AArch64) - Primary, modern
- **Multiboot1/2** (x86_64) - Legacy QEMU support
- **Device Tree** (AArch64) - ARM standard

## Adding a New Boot Protocol
1. Create `boot/newprotocol.rs`
2. Implement `fn parse() -> BootInfo`
3. Add entry point in `boot/mod.rs`
4. Update linker script if needed

## BootInfo Contract
See `BootInfo` struct - this is the ONLY interface.
```

---

## Handoff Checklist

### Code Quality
- [ ] All tests pass (`cargo xtask test behavior`)
- [ ] No compiler warnings
- [ ] No clippy warnings (`cargo clippy`)
- [ ] All `unsafe` documented

### Documentation
- [ ] Boot module has README
- [ ] ARCHITECTURE.md updated
- [ ] Team file complete with findings

### Real Hardware
- [ ] NUC boots successfully (or documented blockers)
- [ ] Serial output captured and saved

### Knowledge Transfer
- [ ] Boot abstraction explained in docs
- [ ] Migration path documented for future protocols
- [ ] Known limitations documented

---

## Final Verification Commands

```bash
# Full build
cargo build --release

# All tests
cargo test --workspace --exclude levitate-kernel
cargo xtask test behavior

# x86_64 boot (Limine)
./run-term.sh

# AArch64 boot
./run-term.sh --aarch64

# Clippy
cargo clippy --workspace -- -D warnings

# Line counts (should be reasonable)
find kernel/src/boot -name "*.rs" | xargs wc -l
```

---

## Success Metrics

| Metric | Target | Measured |
|--------|--------|----------|
| boot.S lines | 0 (or <50 stub) | TBD |
| BootInfo consumers | All init code | TBD |
| Real hardware boot | Yes | TBD |
| Test pass rate | 100% | TBD |
| Unsafe blocks documented | 100% | TBD |

---

## Remaining Work for Future Teams

After this refactor:
1. **NVMe driver** - Boot from NVMe instead of initramfs
2. **SMP boot** - Multi-core initialization via Limine SMP
3. **UEFI runtime** - Access UEFI services after boot
4. **Secure boot** - Sign kernel for UEFI secure boot

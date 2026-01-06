# Phase 3: Fix Design and Validation Plan

**Bug:** Inconsistent error handling across LevitateOS  
**Author:** TEAM_151  
**Status:** Ready for Implementation

---

## Root Cause Summary

No unified error architecture exists. Each module invented its own approach.

---

## Fix Strategy

### Approach: Incremental Migration with Error Codes

Rather than creating a new `levitate-error` crate (complex, risky), we will:

1. **Add error codes to existing enums** - Minimal change, high value
2. **Replace `&'static str` with enums** - One module at a time
3. **Preserve inner errors in conversions** - Wrap, don't discard
4. **Implement `Display` with codes** - `E{code:04X}: {message}`

### Error Code System

```
Format: 0xSSCC
  SS = Subsystem (00-FF)
  CC = Error code within subsystem (00-FF)
```

### Subsystem Allocation (Comprehensive)

Based on ROADMAP.md analysis and standard kernel subsystem needs.

#### Core Kernel (0x00-0x1F)
| Range | Subsystem | Status | Notes |
|-------|-----------|--------|-------|
| `0x00xx` | Generic/Core | Reserved | Catch-all, out-of-memory, etc. |
| `0x01xx` | MMU | **Planned** | Page tables, mapping, TLB |
| `0x02xx` | ELF | **Planned** | Loader errors |
| `0x03xx` | Process/Task | **Planned** | Spawn, exit, scheduling |
| `0x04xx` | Syscall | **Planned** | Internal errors (not errno) |
| `0x05xx` | Filesystem (VFS) | **Planned** | Generic FS layer |
| `0x06xx` | Block Device | ✅ DONE | TEAM_150 |
| `0x07xx` | Network | **Planned** | TCP/IP stack |
| `0x08xx` | GPU/Graphics | Reserved | VirtIO GPU, framebuffer |
| `0x09xx` | FDT/DeviceTree | **Planned** | DTB parsing |
| `0x0Axx` | PCI | Reserved | Bus enumeration, BAR |
| `0x0Bxx` | VirtIO | Reserved | Transport layer |
| `0x0Cxx` | Interrupt/IRQ | Reserved | GIC, IRQ routing |
| `0x0Dxx` | Timer | Reserved | Generic timer, scheduling |
| `0x0Exx` | Console/TTY | Reserved | UART, terminal |
| `0x0Fxx` | Allocator | Reserved | Buddy, slab, heap |

#### Drivers (0x10-0x2F)
| Range | Subsystem | Status | Notes |
|-------|-----------|--------|-------|
| `0x10xx` | UART/Serial | Reserved | PL011, debug console |
| `0x11xx` | Keyboard/Input | Reserved | VirtIO input |
| `0x12xx` | Mouse/Pointer | Reserved | Future input devices |
| `0x13xx` | Storage (NVMe) | Reserved | Future NVMe driver |
| `0x14xx` | USB | Reserved | Future USB stack |
| `0x15xx` | Audio | Reserved | Future audio support |
| `0x16xx` | 9P Filesystem | Reserved | VirtIO 9P (ROADMAP) |
| `0x17xx` | RTC | Reserved | Real-time clock |
| `0x18xx` | Watchdog | Reserved | Hardware watchdog |
| `0x19xx` | Power/PSCI | Reserved | Shutdown, reboot (Phase 12) |
| `0x1Axx` | Crypto | Reserved | Future crypto accelerator |
| `0x1Bxx`-`0x1Fxx` | (Reserved) | — | Future drivers |

#### Userspace Interface (0x20-0x2F)
| Range | Subsystem | Status | Notes |
|-------|-----------|--------|-------|
| `0x20xx` | Process Mgmt | Reserved | fork, exec, wait (Phase 12) |
| `0x21xx` | Signals | Reserved | kill, sigaction (Phase 12) |
| `0x22xx` | IPC | Reserved | pipes, sockets, shared mem |
| `0x23xx` | Permissions | Reserved | UID/GID checks (Phase 16) |
| `0x24xx` | Capabilities | Reserved | CAP_* (Phase 16) |
| `0x25xx`-`0x2Fxx` | (Reserved) | — | Future userspace |

#### Platform-Specific (0x30-0x3F)
| Range | Subsystem | Status | Notes |
|-------|-----------|--------|-------|
| `0x30xx` | QEMU/virt | Reserved | QEMU-specific errors |
| `0x31xx` | Raspberry Pi | Reserved | RPi4/5 (Phase 9) |
| `0x32xx` | Pixel/Tensor | Reserved | GS101 (Phase 9 moonshot) |
| `0x33xx`-`0x3Fxx` | (Reserved) | — | Future platforms |

#### Filesystem-Specific (0x40-0x4F)
| Range | Subsystem | Status | Notes |
|-------|-----------|--------|-------|
| `0x40xx` | FAT32 | Reserved | embedded-sdmmc |
| `0x41xx` | ext4 | Reserved | ext4-view |
| `0x42xx` | initramfs/CPIO | Reserved | Boot ramdisk |
| `0x43xx` | procfs | Reserved | /proc (Phase 12) |
| `0x44xx` | devfs | Reserved | /dev |
| `0x45xx` | tmpfs | Reserved | In-memory FS |
| `0x46xx`-`0x4Fxx` | (Reserved) | — | Future filesystems |

#### Extended (0x50-0xFF)
| Range | Subsystem | Status | Notes |
|-------|-----------|--------|-------|
| `0x50xx`-`0xEFxx` | (Reserved) | — | Future expansion |
| `0xF0xx`-`0xFExx` | Vendor/Custom | — | Platform-specific |
| `0xFFxx` | Debug/Test | Reserved | Test harness only |

### Pattern to Follow (from TEAM_150's BlockError)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsystemError {
    Variant1,
    Variant2,
}

impl SubsystemError {
    pub const fn code(&self) -> u16 {
        match self {
            Self::Variant1 => 0xSSC1,
            Self::Variant2 => 0xSSC2,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Variant1 => "Description 1",
            Self::Variant2 => "Description 2",
        }
    }
}

impl core::fmt::Display for SubsystemError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "E{:04X}: {}", self.code(), self.name())
    }
}

impl core::error::Error for SubsystemError {}
```

---

## Reversal Strategy

If any migration causes problems:

1. **Revert the specific file** - Each module is independent
2. **Keep error codes** - They're additive, non-breaking
3. **Fallback to `&'static str`** - Can always convert back

**Revert signals:**
- Build failures in dependent code
- Test failures after migration
- Runtime panics in new error paths

---

## Test Strategy

### Unit Tests (per error type)

```rust
#[test]
fn test_error_codes_unique() {
    // Verify no duplicate codes within subsystem
}

#[test]
fn test_display_format() {
    let err = MmuError::NotMapped;
    assert_eq!(format!("{}", err), "E0102: Page not mapped");
}
```

### Integration Tests

- Verify error propagation through call chains
- Verify no information loss in conversions

### Regression Tests

- Golden boot test must still pass
- Existing behavior unchanged (just better error messages)

---

## Impact Analysis

### API Changes

| Function | Before | After |
|----------|--------|-------|
| `mmu::map_page()` | `Result<(), &'static str>` | `Result<(), MmuError>` |
| `user_mm::map_user_page()` | `Result<(), &'static str>` | `Result<(), MmuError>` |
| `fs::init()` | `Result<(), &'static str>` | `Result<(), FsError>` |

### Caller Updates Required

Most callers use `?` or `.map_err()` - minimal changes needed.

### Breaking Changes

- Functions returning `&'static str` will return typed errors
- Callers matching on string content will need updates (none found)

---

## Implementation Order

Execute in dependency order (leaf modules first):

1. **ElfError** - Add codes only (no API change)
2. **FdtError** - Add codes only (no API change)
3. **MmuError** - New type, replaces `&'static str` in HAL
4. **ProcessError** - Update to preserve inner errors
5. **FsError** - New type, replaces `&'static str`
6. **NetError** - Add codes only
7. **VirtioError** - Handle DMA panics (if possible)

---

## Exit Criteria for Phase 3

- [x] Fix strategy defined
- [x] Error code system designed
- [x] Pattern established (BlockError)
- [x] Reversal strategy documented
- [x] Test strategy defined
- [x] Implementation order determined

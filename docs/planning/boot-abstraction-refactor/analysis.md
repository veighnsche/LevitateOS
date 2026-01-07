# x86_64 Boot Architecture Analysis & Redesign

## TEAM_279: Design Review

### Target Hardware
- **Device**: Intel NUC i3 7th Gen
- **RAM**: 32GB
- **Storage**: 1TB NVMe
- **Firmware**: UEFI (no legacy BIOS)

---

## Current State: "Patch on Patch"

### What Was Implemented
| Component | Status | Issues |
|-----------|--------|--------|
| Multiboot1 header | ✅ Working | Only for QEMU `-kernel` |
| Multiboot2 header | ✅ Present | Never tested with GRUB |
| boot.S (32→64 transition) | ✅ Working | Complex, manual page tables |
| APIC mapping | ✅ Fixed (TEAM_278) | Was missing, caused crash |
| Multiboot arg preservation | ✅ Fixed (TEAM_278) | Registers clobbered by BSS clear |

### The Real Problem
**The current boot path is designed for QEMU testing, NOT for real hardware.**

1. **Multiboot1/2 are legacy protocols** - Modern x86_64 systems boot via UEFI
2. **We're doing the bootloader's job** - Page table setup, long mode transition
3. **No UEFI support** - The NUC uses UEFI, not legacy BIOS
4. **No NVMe driver** - Can't boot from the 1TB NVMe without UEFI or custom driver

---

## Boot Protocol Options for x86_64

### Option 1: Multiboot2 via GRUB (Current Approach)
```
UEFI → GRUB2 → Multiboot2 → boot.S (32-bit) → Long Mode → kernel_main
```
**Pros:**
- Works in QEMU
- GRUB handles UEFI complexity

**Cons:**
- Requires GRUB installation
- We still do 32→64 transition manually
- Complex boot.S with page table setup
- Multiboot2 info parsing is incomplete

### Option 2: UEFI Direct (via `uefi-rs` crate)
```
UEFI → EFI Application (our kernel) → Already in 64-bit → kernel_main
```
**Pros:**
- Already in 64-bit mode
- UEFI provides memory map, framebuffer
- No bootloader needed (kernel IS the EFI app)
- Access to UEFI Runtime Services

**Cons:**
- Must link as PE32+ executable
- Need UEFI-specific entry point
- Different from AArch64 path (DTB-based)

### Option 3: Limine Boot Protocol (RECOMMENDED)
```
UEFI → Limine → Limine Protocol → kernel (64-bit) → kernel_main
```
**Pros:**
- **Modern, well-documented protocol**
- Boots via UEFI or legacy BIOS
- Kernel receives CPU already in 64-bit long mode
- Provides: memory map, framebuffer, RSDP, SMP info
- **Supports both x86_64 AND AArch64** (unified abstraction!)
- Simple: just implement `limine_requests` in kernel
- Active development, good Rust support (`limine` crate)

**Cons:**
- Requires Limine bootloader (trivial to install)
- Newer, less "standard" than GRUB

### Option 4: Linux Boot Protocol (via GRUB or EFI stub)
```
UEFI → GRUB/EFI Stub → Linux Boot Protocol → bzImage-like kernel
```
**Pros:**
- Industry standard
- Well-documented

**Cons:**
- Very complex protocol
- Designed for Linux, overkill for our needs

---

## Recommendation: Limine Protocol

### Why Limine?

1. **Unified Abstraction** - Same protocol works for x86_64 AND AArch64
2. **No Assembly Needed** - Limine does the 32→64 transition for us
3. **Rich Boot Info** - Memory map, framebuffer, ACPI tables, kernel modules
4. **NVMe Support** - Limine can load kernel from NVMe via UEFI
5. **Simple Integration** - Just add `limine` crate and define requests
6. **Real Hardware Ready** - Works on actual UEFI systems

### Architecture with Limine

```
┌─────────────────────────────────────────────────────────────────┐
│                        FIRMWARE LAYER                           │
├─────────────────────────────────────────────────────────────────┤
│  x86_64 NUC: UEFI                 │  AArch64: U-Boot/ABL        │
└─────────────────────────────────────────────────────────────────┘
                    │                              │
                    ▼                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      BOOTLOADER LAYER                           │
├─────────────────────────────────────────────────────────────────┤
│  Limine (x86_64 + AArch64)        │  Direct boot (QEMU -kernel) │
└─────────────────────────────────────────────────────────────────┘
                    │                              │
                    ▼                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   BOOT INFO ABSTRACTION                         │
├─────────────────────────────────────────────────────────────────┤
│                      BootInfo struct                            │
│  - memory_map: &[MemoryRegion]                                  │
│  - framebuffer: Option<Framebuffer>                             │
│  - rsdp: Option<*const u8>        (ACPI for x86_64)             │
│  - dtb: Option<*const u8>         (DTB for AArch64)             │
│  - cmdline: &str                                                │
│  - initramfs: Option<&[u8]>                                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        KERNEL ENTRY                             │
│                  kernel_main(boot_info: &BootInfo)              │
└─────────────────────────────────────────────────────────────────┘
```

### What This Removes

With Limine, we can **delete**:
- `kernel/src/arch/x86_64/boot.S` (all 330 lines!)
- Multiboot1/2 headers
- Manual GDT64 setup
- Manual page table setup in assembly
- 32-bit to 64-bit transition code

### What This Adds

```rust
// kernel/src/boot/limine.rs
use limine::*;

#[used]
static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);

#[used]
static MEMORY_MAP: LimineMemmapRequest = LimineMemmapRequest::new(0);

#[used]
static FRAMEBUFFER: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

#[used]
static RSDP: LimineRsdpRequest = LimineRsdpRequest::new(0);  // x86_64 ACPI

// Entry point - Limine calls this in 64-bit mode
#[no_mangle]
extern "C" fn _start() -> ! {
    let boot_info = BootInfo::from_limine();
    kernel_main(&boot_info);
}
```

---

## Migration Path

### Phase 1: Add Limine Support (Parallel Path)
1. Add `limine` crate dependency
2. Create `kernel/src/boot/limine.rs` with Limine requests
3. Create `BootInfo` abstraction struct
4. Create Limine entry point that builds `BootInfo`
5. Test on QEMU with Limine ISO

### Phase 2: Unify AArch64 to Same Abstraction
1. Create `kernel/src/boot/dtb.rs` for AArch64
2. Parse DTB into `BootInfo` struct
3. Both architectures now use same `kernel_main(&BootInfo)` signature

### Phase 3: Remove Legacy Boot Code
1. Delete `boot.S` multiboot code (keep for reference in git)
2. Remove multiboot1/2 headers
3. Update build system to produce Limine-compatible ELF

### Phase 4: Real Hardware Testing
1. Create bootable USB with Limine + LevitateOS
2. Test on Intel NUC
3. Add NVMe support via UEFI runtime or custom driver

---

## Immediate Actions

### Keep for Now (QEMU Development)
- Current multiboot1 path works for `qemu -kernel`
- Useful for quick iteration

### Add in Parallel
- Limine boot path for real hardware
- `BootInfo` abstraction

### Delete Eventually
- All of `boot.S` except maybe a stub
- Multiboot header sections in linker script

---

## Questions for User

1. **Do you want to proceed with Limine integration?**
   - This is the cleanest path to real NUC hardware

2. **Should we keep QEMU multiboot path for development?**
   - Limine works in QEMU too, but multiboot is faster to iterate

3. **Timeline priority?**
   - Real NUC boot soon, or focus on other features first?

---

## References

- [Limine Boot Protocol](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md)
- [limine-rs crate](https://crates.io/crates/limine)
- [Limine Bare Bones](https://wiki.osdev.org/Limine_Bare_Bones)
- [Theseus OS UEFI approach](https://github.com/theseus-os/Theseus) (in .external-kernels)

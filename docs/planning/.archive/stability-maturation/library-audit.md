# Library Audit: Hand-Rolled Code vs Existing Crates

**TEAM_311**: Stability Maturation
**Date**: 2026-01-08

This document identifies code that could be replaced with battle-tested existing crates.

---

## Summary

| Category | Current | Recommended Crate | Priority |
|----------|---------|-------------------|----------|
| Syscall Numbers | Hand-rolled enum | `linux-raw-sys` (already used in userspace) | ✅ Done |
| ELF Parsing | `kernel/src/loader/elf.rs` | `goblin` or `elf` | HIGH |
| CPIO Parsing | `crates/utils/src/cpio.rs` | `cpio` crate | MEDIUM |
| Ring Buffer | `crates/utils/RingBuffer` | `heapless::spsc::Queue` | LOW |
| Allocators | Hand-rolled Buddy+Slab | Keep (kernel-specific) | N/A |
| Page Tables | Hand-rolled | Keep (kernel-specific) | N/A |
| GDT/IDT | Hand-rolled | `x86_64` crate (already dep) | HIGH |
| Multiboot2 | `crates/hal/multiboot2.rs` | `multiboot2` crate | MEDIUM |
| ANSI Terminal | `crates/term/` | `vte` or keep | LOW |
| VirtIO | Custom VirtQueue | `virtio-drivers` (already used) | ✅ Done |
| PCI | Using `pci-types` | ✅ Already using | ✅ Done |
| FDT | Using `fdt` crate | ✅ Already using | ✅ Done |
| Bitflags | Using `bitflags` | ✅ Already using | ✅ Done |
| Spinlock | Using `spin` crate | ✅ Already using | ✅ Done |
| HashMap | Using `hashbrown` | ✅ Already using | ✅ Done |

---

## HIGH Priority Replacements

### 1. ELF Parsing → `goblin` or `elf`

**Current**: `kernel/src/loader/elf.rs` (~300 lines hand-rolled)

**Problem**: Manual struct parsing, easy to get wrong, no validation

**Recommended**: [`goblin`](https://crates.io/crates/goblin) or [`elf`](https://crates.io/crates/elf)

```toml
# Cargo.toml
goblin = { version = "0.8", default-features = false, features = ["elf64"] }
```

**Benefits**:
- Battle-tested ELF parsing
- Handles edge cases
- `no_std` compatible
- Supports both ELF32 and ELF64

**Migration**:
```rust
// Before (hand-rolled)
let header = Elf64Header::parse(data)?;

// After (goblin)
use goblin::elf::Elf;
let elf = Elf::parse(data)?;
```

---

### 2. GDT/IDT → Use `x86_64` crate features

**Current**: `crates/hal/src/x86_64/gdt.rs`, `idt.rs` (hand-rolled)

**Problem**: Already have `x86_64 = "0.15"` as dependency but not using its GDT/IDT!

**The `x86_64` crate provides**:
- `x86_64::structures::gdt::GlobalDescriptorTable`
- `x86_64::structures::idt::InterruptDescriptorTable`
- `x86_64::structures::tss::TaskStateSegment`

**Migration**:
```rust
// Before (hand-rolled)
pub struct Gdt { ... }
pub struct Idt([IdtEntry; 256]);

// After (x86_64 crate)
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::idt::InterruptDescriptorTable;
```

---

## MEDIUM Priority Replacements

### 3. Multiboot2 → `multiboot2` crate

**Current**: `crates/hal/src/x86_64/multiboot2.rs` (~200 lines)

**Recommended**: [`multiboot2`](https://crates.io/crates/multiboot2)

```toml
multiboot2 = "0.20"
```

**Benefits**:
- Complete tag parsing
- Memory-safe iteration
- Well-maintained

---

### 4. CPIO Parsing → `cpio` crate

**Current**: `crates/utils/src/cpio.rs` (~400 lines)

**Recommended**: [`cpio`](https://crates.io/crates/cpio)

```toml
cpio = { version = "0.4", default-features = false }
```

**Note**: Check `no_std` compatibility first.

---

## LOW Priority / Keep As-Is

### 5. Ring Buffer

**Current**: `crates/utils/RingBuffer` (70 lines)

**Options**:
- [`heapless`](https://crates.io/crates/heapless) - `spsc::Queue` for single-producer single-consumer
- [`ringbuffer`](https://crates.io/crates/ringbuffer)

**Verdict**: Current implementation is simple and works. Low priority.

---

### 6. ANSI Terminal Parser

**Current**: `crates/term/` (~200 lines)

**Options**:
- [`vte`](https://crates.io/crates/vte) - Full VT100/ANSI parser
- [`ansi-parser`](https://crates.io/crates/ansi-parser)

**Verdict**: Current implementation handles our needs. Future improvement.

---

### 7. Allocators (Buddy + Slab)

**Current**: `crates/hal/src/allocator/`

**Verdict**: KEEP. Kernel allocators are tightly coupled to memory layout. Standard crates don't fit bare-metal kernel requirements.

---

### 8. Page Tables / MMU

**Current**: `crates/hal/src/*/mmu.rs`, `paging.rs`

**Verdict**: KEEP. Architecture-specific, tightly coupled to kernel memory model.

---

## Already Using Good Crates ✅

| Crate | Usage |
|-------|-------|
| `virtio-drivers` | VirtIO device drivers |
| `embedded-graphics` | GPU framebuffer |
| `bitflags` | Flag enums |
| `spin` | Spinlocks, RwLock, Once, Lazy |
| `hashbrown` | no_std HashMap |
| `x86_64` | CPU features (but not GDT/IDT!) |
| `aarch64-cpu` | ARM CPU features |
| `fdt` | Device tree parsing |
| `acpi` + `aml` | ACPI parsing |
| `linux-raw-sys` | Linux ABI constants |

---

## Action Items

### Immediate (This Sprint)
1. [ ] **Use `x86_64` crate's GDT/IDT** instead of hand-rolled
2. [ ] **Evaluate `goblin` for ELF parsing** - check no_std support

### Next Sprint
3. [ ] **Replace multiboot2 parsing** with `multiboot2` crate
4. [ ] **Evaluate CPIO crate** for initramfs parsing

### Future
5. [ ] Consider `heapless` for ring buffer if issues arise
6. [ ] Consider `vte` for terminal if ANSI compliance needed

---

## References

- [Awesome Embedded Rust](https://github.com/rust-embedded/awesome-embedded-rust)
- [no_std crates](https://crates.io/categories/no-std)
- [OS Dev Wiki](https://wiki.osdev.org/)

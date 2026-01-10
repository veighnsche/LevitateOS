# ELF Loader: Best Practices from External Kernels

**TEAM_352** | 2026-01-09

---

## Executive Summary

After reviewing external kernels and existing planning docs:

1. **Use `goblin` crate** for ELF parsing (HIGH priority per library-audit.md)
2. **Theseus** provides excellent relocation handling patterns
3. **Current hand-rolled parser** should be replaced

---

## 1. Goblin Crate Recommendation

### From `docs/planning/stability-maturation/library-audit.md`

```
| Category | Current | Recommended Crate | Priority |
|----------|---------|-------------------|----------|
| ELF Parsing | kernel/src/loader/elf.rs | `goblin` or `elf` | HIGH |
```

### Why Goblin?

- **Battle-tested** — Widely used in binary analysis tools
- **no_std compatible** — Works in kernel context
- **Handles edge cases** — Less bugs than hand-rolled parsing
- **Both ELF32/64** — Future-proof
- **Relocation constants** — `goblin::elf::reloc::*` provides all R_* constants

### Cargo.toml Addition

```toml
[dependencies]
goblin = { version = "0.8", default-features = false, features = ["elf64", "alloc"] }
```

---

## 2. Theseus Patterns

### 2.1 Relocation Constants from Goblin

```rust
// From theseus/kernel/crate_metadata/src/lib.rs
use goblin::elf::reloc::*;

// Provides:
// R_X86_64_64, R_X86_64_PC32, R_X86_64_PLT32, etc.
// R_AARCH64_ABS64, R_AARCH64_RELATIVE, etc.
```

### 2.2 Relocation Entry Abstraction

```rust
// Theseus uses a RelocationEntry struct
pub struct RelocationEntry {
    pub typ: u32,       // Relocation type (R_AARCH64_RELATIVE, etc.)
    pub offset: usize,  // Where to apply
    pub addend: usize,  // Value to add
}

impl RelocationEntry {
    pub fn from_elf_relocation(rela: &Rela) -> Self {
        Self {
            typ: rela.r_type(),
            offset: rela.r_offset as usize,
            addend: rela.r_addend as usize,
        }
    }
}
```

### 2.3 Architecture-Specific write_relocation

```rust
// Theseus pattern: arch-specific relocation handlers
#[cfg(target_arch = "x86_64")]
fn write_relocation_arch(entry: RelocationEntry, ...) -> Result<(), &'static str> {
    match entry.typ {
        R_X86_64_64 => { /* absolute 64-bit */ }
        R_X86_64_PC32 | R_X86_64_PLT32 => { /* PC-relative 32-bit */ }
        R_X86_64_RELATIVE => { /* load_base + addend */ }
        other => return Err("unsupported relocation"),
    }
    Ok(())
}

#[cfg(target_arch = "aarch64")]
fn write_relocation_arch(entry: RelocationEntry, ...) -> Result<(), &'static str> {
    match entry.typ {
        R_AARCH64_ABS64 => { /* absolute 64-bit */ }
        R_AARCH64_RELATIVE => { /* load_base + addend */ }
        other => return Err("unsupported relocation"),
    }
    Ok(())
}
```

---

## 3. Using Goblin for ELF Parsing

### 3.1 Basic Parsing

```rust
use goblin::elf::Elf;

pub fn parse_elf(data: &[u8]) -> Result<ElfInfo, ElfError> {
    let elf = Elf::parse(data).map_err(|_| ElfError::InvalidFormat)?;
    
    // Check type
    let is_pie = elf.header.e_type == goblin::elf::header::ET_DYN;
    
    // Get program headers
    for phdr in &elf.program_headers {
        if phdr.p_type == goblin::elf::program_header::PT_LOAD {
            // Load segment
        }
    }
    
    // Get dynamic section
    if let Some(dynamic) = &elf.dynamic {
        // Process dynamic entries
    }
    
    Ok(ElfInfo { ... })
}
```

### 3.2 Accessing Relocations

```rust
use goblin::elf::reloc::Rela;

// Goblin parses relocations automatically
for rela in &elf.dynrelas {
    let r_type = rela.r_type;
    let r_offset = rela.r_offset;
    let r_addend = rela.r_addend;
    
    apply_relocation(load_base, r_type, r_offset, r_addend)?;
}
```

### 3.3 ET_DYN / PIE Detection

```rust
use goblin::elf::header::ET_DYN;

let is_pie = elf.header.e_type == ET_DYN;
let load_base = if is_pie { 0x10000 } else { 0 };
```

---

## 4. Recommended Implementation Approach

### Option A: Replace elf.rs with Goblin (Recommended)

**Pros:**
- Cleaner code
- Better validation
- Access to all relocation constants
- Future-proof

**Cons:**
- Larger change
- Need to verify no_std works

### Option B: Keep elf.rs, Add Goblin for Relocations Only

**Pros:**
- Smaller change
- Less risk

**Cons:**
- Two parsing systems
- Maintenance burden

### Recommendation: **Option A**

Replace the hand-rolled ELF parser with goblin. The library audit already recommends this as HIGH priority.

---

## 5. Updated Implementation Plan

### Step 1: Add Goblin Dependency

```toml
# crates/kernel/Cargo.toml
goblin = { version = "0.8", default-features = false, features = ["elf64", "alloc"] }
```

### Step 2: Refactor elf.rs to Use Goblin

```rust
// New elf.rs
use goblin::elf::{Elf, program_header::*, header::*};
use goblin::elf::reloc::*;

pub fn load_elf(data: &[u8], ttbr0_phys: usize) -> Result<(usize, usize), ElfError> {
    let elf = Elf::parse(data).map_err(|_| ElfError::InvalidFormat)?;
    
    // Validate architecture
    #[cfg(target_arch = "aarch64")]
    if elf.header.e_machine != EM_AARCH64 {
        return Err(ElfError::WrongArchitecture);
    }
    
    #[cfg(target_arch = "x86_64")]
    if elf.header.e_machine != EM_X86_64 {
        return Err(ElfError::WrongArchitecture);
    }
    
    // Calculate load base
    let load_base = if elf.header.e_type == ET_DYN { 0x10000 } else { 0 };
    
    // Load segments
    for phdr in &elf.program_headers {
        if phdr.p_type == PT_LOAD {
            load_segment(ttbr0_phys, load_base, phdr, data)?;
        }
    }
    
    // Process relocations for PIE
    if elf.header.e_type == ET_DYN {
        for rela in &elf.dynrelas {
            apply_relocation(ttbr0_phys, load_base, rela)?;
        }
    }
    
    let entry_point = load_base + elf.header.e_entry as usize;
    Ok((entry_point, brk))
}
```

### Step 3: Implement apply_relocation Using Goblin Constants

```rust
fn apply_relocation(
    ttbr0_phys: usize,
    load_base: usize,
    rela: &Rela,
) -> Result<(), ElfError> {
    let r_type = rela.r_type;
    let target_va = load_base + rela.r_offset as usize;
    
    match r_type {
        #[cfg(target_arch = "aarch64")]
        R_AARCH64_RELATIVE => {
            let value = (load_base as i64 + rela.r_addend) as u64;
            write_user_u64(ttbr0_phys, target_va, value)?;
        }
        
        #[cfg(target_arch = "x86_64")]
        R_X86_64_RELATIVE => {
            let value = (load_base as i64 + rela.r_addend) as u64;
            write_user_u64(ttbr0_phys, target_va, value)?;
        }
        
        _ => {
            log::warn!("[ELF] Unsupported relocation type: {}", r_type);
        }
    }
    
    Ok(())
}
```

---

## 6. Testing Strategy

1. **Verify goblin compiles in no_std** context
2. **Test ET_EXEC binaries** still work (regression)
3. **Test PIE binaries** (eyra-hello)
4. **Compare relocation counts** with readelf output

---

## 7. References

| Resource | URL |
|----------|-----|
| Goblin crate | https://crates.io/crates/goblin |
| Goblin docs | https://docs.rs/goblin |
| Theseus crate_metadata | `.external-kernels/theseus/kernel/crate_metadata/` |
| LevitateOS library audit | `docs/planning/stability-maturation/library-audit.md` |
| ARM64 relocations | https://github.com/ARM-software/abi-aa/blob/main/aaelf64/aaelf64.rst |

# Phase 2: Design — ELF Dynamic Loader

**TEAM_352** | ELF Loader Feature  
**Created:** 2026-01-09  
**Depends on:** Phase 1 (Discovery)

---

## 0. Key Decision: Use Goblin Crate

**Per `docs/planning/stability-maturation/library-audit.md`:**

The current hand-rolled ELF parser should be replaced with `goblin` (HIGH priority).

```toml
goblin = { version = "0.8", default-features = false, features = ["elf64", "alloc"] }
```

**Benefits:**
- Battle-tested parsing
- Provides all relocation constants (`R_AARCH64_RELATIVE`, etc.)
- `no_std` compatible with `alloc` feature
- Used by Theseus kernel for relocations

**See:** `docs/planning/elf-loader/best-practices.md` for full details.

---

## 1. Proposed Solution

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    spawn_from_elf()                         │
├─────────────────────────────────────────────────────────────┤
│  1. Parse ELF header                                        │
│  2. Determine ELF type (ET_EXEC or ET_DYN)                 │
│  3. Calculate load base (0 for ET_EXEC, chosen for ET_DYN) │
│  4. Load PT_LOAD segments (adjusted by load_base)          │
│  5. If ET_DYN: Find PT_DYNAMIC, process relocations        │
│  6. Calculate entry point (load_base + e_entry)            │
│  7. Set up stack, create task                              │
└─────────────────────────────────────────────────────────────┘
```

### Key Changes

1. **Accept ET_DYN** in header validation
2. **Choose load base** for PIE binaries
3. **Process PT_DYNAMIC** segment to find relocations
4. **Apply relocations** before returning entry point

---

## 2. Detailed Design

> **Best Practices Reference:** See `docs/planning/elf-loader/best-practices.md` for Theseus patterns and goblin usage.

### 2.1 ELF Type Handling (Using Goblin)

```rust
use goblin::elf::{Elf, header::{ET_EXEC, ET_DYN}};

/// Parse and validate ELF type
fn parse_elf(data: &[u8]) -> Result<Elf, ElfError> {
    let elf = Elf::parse(data).map_err(|_| ElfError::InvalidFormat)?;
    
    // Validate type (goblin provides constants)
    match elf.header.e_type {
        ET_EXEC | ET_DYN => Ok(elf),
        _ => Err(ElfError::NotExecutable),
    }
}
```

### 2.2 Load Base Selection

**Decision:** Use fixed load base (not random) for simplicity.

```rust
use goblin::elf::header::{ET_EXEC, ET_DYN};

/// Calculate load base for this ELF.
/// - ET_EXEC: 0 (addresses are absolute)
/// - ET_DYN: 0x10000 (fixed base, simple and debuggable)
fn calculate_load_base(elf: &Elf) -> usize {
    if elf.header.e_type == ET_DYN {
        0x10000  // 64KB, above null page
    } else {
        0  // ET_EXEC uses absolute addresses
    }
}
```

**Rationale:**
- ASLR can be added later
- Fixed base simplifies debugging
- 0x10000 avoids null pointer region

### 2.3 Segment Loading with Base Offset

```rust
// Modified load() function
pub fn load(&self, ttbr0_phys: usize) -> Result<(usize, usize), ElfError> {
    let load_base = self.load_base();
    
    for phdr in self.program_headers() {
        if !phdr.is_loadable() {
            continue;
        }
        
        // Apply load base to virtual address
        let vaddr = load_base + phdr.vaddr();
        // ... rest of loading logic with adjusted vaddr
    }
    
    // Process relocations for ET_DYN
    if self.header.e_type == ET_DYN {
        self.process_relocations(ttbr0_phys, load_base)?;
    }
    
    // Return adjusted entry point
    let entry_point = load_base + self.header.e_entry as usize;
    Ok((entry_point, initial_brk))
}
```

### 2.4 Relocation Processing (Using Goblin)

> **Key Insight:** Goblin parses all dynamic sections and relocations automatically.
> No need for manual PT_DYNAMIC parsing or Elf64Dyn/Elf64Rela structs.

#### 2.4.1 Accessing Relocations via Goblin

Goblin provides parsed relocations directly:

```rust
use goblin::elf::Elf;
use goblin::elf::reloc::*;  // Provides R_AARCH64_RELATIVE, R_X86_64_RELATIVE, etc.

/// Process all relocations for a PIE binary
fn process_relocations(
    elf: &Elf,
    data: &[u8],
    ttbr0_phys: usize,
    load_base: usize,
) -> Result<usize, ElfError> {
    let mut count = 0;
    
    // Goblin parses .rela.dyn automatically into dynrelas
    for rela in &elf.dynrelas {
        apply_relocation(ttbr0_phys, load_base, &rela)?;
        count += 1;
    }
    
    log::debug!("[ELF] Applied {} relocations", count);
    Ok(count)
}
```

#### 2.4.2 Architecture-Specific Relocation Handlers (Theseus Pattern)

Following Theseus kernel patterns from `best-practices.md`:

```rust
use goblin::elf::reloc::*;

/// Apply a single relocation using goblin's Rela type
fn apply_relocation(
    ttbr0_phys: usize,
    load_base: usize,
    rela: &goblin::elf::reloc::Rela,
) -> Result<(), ElfError> {
    let target_va = load_base + rela.r_offset as usize;
    
    match rela.r_type {
        // aarch64 relocations (goblin provides these constants)
        #[cfg(target_arch = "aarch64")]
        R_AARCH64_RELATIVE => {
            let value = (load_base as i64 + rela.r_addend) as u64;
            write_user_u64(ttbr0_phys, target_va, value)?;
        }
        
        // x86_64 relocations
        #[cfg(target_arch = "x86_64")]
        R_X86_64_RELATIVE => {
            let value = (load_base as i64 + rela.r_addend) as u64;
            write_user_u64(ttbr0_phys, target_va, value)?;
        }
        
        other => {
            log::trace!("[ELF] Skipping unsupported reloc type {}", other);
        }
    }
    
    Ok(())
}

fn write_user_u64(ttbr0_phys: usize, va: usize, value: u64) -> Result<(), ElfError> {
    let l0_va = mmu::phys_to_virt(ttbr0_phys);
    let l0 = unsafe { &mut *(l0_va as *mut mmu::PageTable) };
    
    let page_va = va & !0xFFF;
    let page_offset = va & 0xFFF;
    
    if let Ok(walk) = mmu::walk_to_entry(l0, page_va, 3, false) {
        let phys = walk.table.entry(walk.index).address() + page_offset;
        let ptr = mmu::phys_to_virt(phys) as *mut u64;
        unsafe { ptr.write_unaligned(value); }
        Ok(())
    } else {
        Err(ElfError::MappingFailed)
    }
}
```

---

## 3. API Design

### Modified Public API

```rust
// elf.rs - Extended error types
define_kernel_error! {
    pub enum ElfError(0x02) {
        // ... existing errors ...
        
        /// Dynamic section not found in PIE binary
        NoDynamicSection = 0x0A => "No dynamic section",
        /// Relocation failed
        RelocationFailed = 0x0B => "Relocation failed",
    }
}

// elf.rs - Extended Elf struct
impl<'a> Elf<'a> {
    /// Check if this is a position-independent executable
    pub fn is_pie(&self) -> bool {
        self.header.e_type == ET_DYN
    }
    
    /// Get the load base for this ELF
    pub fn load_base(&self) -> usize;
    
    /// Load the ELF and process relocations
    pub fn load(&self, ttbr0_phys: usize) -> Result<(usize, usize), ElfError>;
}
```

---

## 4. Behavioral Decisions

### Q1: Load address selection

**Decision:** Fixed load base at 0x10000 for ET_DYN.

**Rationale:**
- Simpler to implement and debug
- ASLR is a security feature, not required for functionality
- Can add randomization later

### Q2: Unsupported relocations

**Decision:** Log warning and continue.

**Rationale:**
- Most PIE binaries only use R_*_RELATIVE
- Failing on first unsupported reloc would break unnecessarily
- Logging helps debugging

### Q3: Architecture abstraction

**Decision:** Use `#[cfg(target_arch)]` for relocation constants.

**Rationale:**
- Simple and clear
- Only a few constants differ
- No need for trait abstraction

### Q4: PT_INTERP handling

**Decision:** Ignore PT_INTERP silently.

**Rationale:**
- We ARE the dynamic linker
- No need to load ld-linux.so
- Binary will work without interpreter

### Q5: Testing strategy

**Decision:** Multi-layered testing.

1. **Unit tests:** Relocation math correctness
2. **Integration test:** Load eyra-hello, verify output
3. **Regression test:** Existing ET_EXEC binaries still work

---

## 5. Implementation Steps

### Step 1: Add ET_DYN Support (Small)

- Add `ET_DYN` constant
- Modify header validation to accept ET_DYN
- Add `is_pie()` method

### Step 2: Implement Load Base (Small)

- Add `load_base()` method
- Modify segment loading to use load_base offset
- Adjust entry point calculation

### Step 3: Add Dynamic Parsing (Medium)

- Add PT_DYNAMIC, DT_* constants
- Add Elf64Dyn, Elf64Rela structs
- Implement `find_dynamic()` and `parse_dynamic()`

### Step 4: Implement Relocations (Medium)

- Add R_*_RELATIVE constants
- Implement `apply_relocation()`
- Add `write_user_u64()` helper
- Call relocation processing after segment loading

### Step 5: Integration & Testing (Medium)

- Build eyra-hello for LevitateOS
- Add to initramfs
- Boot and test
- Add regression tests

---

## 6. File Changes Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `crates/kernel/src/loader/elf.rs` | Modify | Add ET_DYN, relocations |
| `crates/kernel/src/loader/mod.rs` | Modify | Export new types if needed |
| `crates/kernel/src/task/process.rs` | Minor | May need auxv updates for load base |

---

## 7. Estimated Effort

| Step | Effort | Risk |
|------|--------|------|
| Step 1: ET_DYN support | 30 min | Low |
| Step 2: Load base | 30 min | Low |
| Step 3: Dynamic parsing | 1 hour | Medium |
| Step 4: Relocations | 1-2 hours | Medium |
| Step 5: Testing | 1-2 hours | Medium |

**Total: 4-6 hours**

---

## 8. Open Questions (Answered)

| # | Question | Answer |
|---|----------|--------|
| 1 | Load address selection | Fixed at 0x10000 |
| 2 | Unsupported relocations | Warn and continue |
| 3 | Architecture abstraction | Use cfg(target_arch) |
| 4 | PT_INTERP handling | Ignore silently |
| 5 | Testing strategy | Unit + integration + regression |

---

## 9. Next Phase

**Phase 3: Implementation** will execute the steps defined above.

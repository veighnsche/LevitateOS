# Phase 3: Implementation — ELF Dynamic Loader

**TEAM_352** | ELF Loader Feature  
**Created:** 2026-01-09  
**Depends on:** Phase 2 (Design)

---

## 1. Implementation Overview

> **Best Practices Reference:** See `docs/planning/elf-loader/best-practices.md` for patterns and goblin usage.

This phase implements the ELF dynamic loader in 5 steps using the **goblin** crate:

| Step | Description | Effort |
|------|-------------|--------|
| 1 | Add goblin dependency & refactor elf.rs | 1 hour |
| 2 | Implement load base for PIE | 30 min |
| 3 | Use goblin for relocation processing | 1 hour |
| 4 | Write architecture-specific handlers | 30 min |
| 5 | Integration testing | 1-2 hours |

---

## 2. Step 1: Add Goblin Dependency & Refactor elf.rs

### 2.1 Tasks

1. Add goblin dependency to `crates/kernel/Cargo.toml`
2. Refactor `elf.rs` to use goblin for parsing
3. Remove hand-rolled Elf64Header parsing (keep struct for tests if needed)

### 2.2 Code Changes

**File:** `crates/kernel/Cargo.toml`

```toml
[dependencies]
goblin = { version = "0.8", default-features = false, features = ["elf64", "alloc"] }
```

**File:** `crates/kernel/src/loader/elf.rs`

```rust
use goblin::elf::{Elf as GoblinElf, header::{ET_EXEC, ET_DYN, EM_AARCH64, EM_X86_64}};
use goblin::elf::program_header::PT_LOAD;

/// Parsed ELF wrapper using goblin
pub struct Elf<'a> {
    inner: GoblinElf<'a>,
    data: &'a [u8],
}

impl<'a> Elf<'a> {
    /// Parse an ELF file from raw bytes using goblin
    pub fn parse(data: &'a [u8]) -> Result<Self, ElfError> {
        let inner = GoblinElf::parse(data).map_err(|_| ElfError::InvalidFormat)?;
        
        // Validate type
        if inner.header.e_type != ET_EXEC && inner.header.e_type != ET_DYN {
            return Err(ElfError::NotExecutable);
        }
        
        // Validate architecture
        #[cfg(target_arch = "aarch64")]
        if inner.header.e_machine != EM_AARCH64 {
            return Err(ElfError::WrongArchitecture);
        }
        
        #[cfg(target_arch = "x86_64")]
        if inner.header.e_machine != EM_X86_64 {
            return Err(ElfError::WrongArchitecture);
        }
        
        Ok(Self { inner, data })
    }
    
    /// Check if this is a position-independent executable (PIE)
    pub fn is_pie(&self) -> bool {
        self.inner.header.e_type == ET_DYN
    }
}
```

### 2.3 Exit Criteria

- Goblin dependency compiles in no_std context
- `Elf::parse()` accepts both ET_EXEC and ET_DYN binaries
- `Elf::is_pie()` returns true for PIE binaries
- Architecture validation uses goblin constants

---

## 3. Step 2: Implement Load Base for PIE

### 3.1 Tasks

1. Add `load_base()` method returning non-zero for PIE
2. Update segment loading to use load base offset
3. Adjust entry point calculation

### 3.2 Code Changes

**File:** `crates/kernel/src/loader/elf.rs`

```rust
impl<'a> Elf<'a> {
    /// Get the base address where the ELF is loaded.
    /// - ET_EXEC: 0 (addresses are absolute)
    /// - ET_DYN: 0x10000 (fixed base for PIE)
    pub fn load_base(&self) -> usize {
        if self.is_pie() {
            0x10000  // 64KB, above null page
        } else {
            0  // ET_EXEC uses absolute addresses
        }
    }
    
    /// Get the entry point adjusted by load base
    pub fn entry_point(&self) -> usize {
        self.load_base() + self.inner.header.e_entry as usize
    }
}
```

In the `load()` function, apply load_base to segment addresses:

```rust
pub fn load(&self, ttbr0_phys: usize) -> Result<(usize, usize), ElfError> {
    let load_base = self.load_base();
    
    for phdr in &self.inner.program_headers {
        if phdr.p_type != PT_LOAD {
            continue;
        }
        
        // Apply load base to virtual address
        let vaddr = load_base + phdr.p_vaddr as usize;
        // ... rest of loading logic with adjusted vaddr
    }
    
    // Return adjusted entry point
    Ok((self.entry_point(), initial_brk))
}
```

### 3.3 Exit Criteria

- PIE segments loaded at load_base + vaddr
- Entry point correctly adjusted for PIE
- ET_EXEC behavior unchanged (load_base = 0)

---

## 4. Step 3: Use Goblin for Relocation Processing

### 4.1 Tasks

**Goblin handles all dynamic section parsing automatically!**

No need to:
- ~~Add PT_DYNAMIC constant~~ (goblin has it)
- ~~Add Elf64Dyn struct~~ (goblin parses this)
- ~~Add DT_* tag constants~~ (goblin has them)
- ~~Implement find_dynamic()~~ (goblin exposes `elf.dynamic`)
- ~~Implement parse_dynamic()~~ (goblin exposes `elf.dynrelas`)

### 4.2 Goblin Already Provides

```rust
// Goblin's Elf struct exposes:
elf.dynamic        // Option<Dynamic> - parsed dynamic section
elf.dynrelas       // Vec<Rela> - all .rela.dyn relocations
elf.pltrelocs      // RelocSection - PLT relocations
```

### 4.3 Code Changes

**File:** `crates/kernel/src/loader/elf.rs`

```rust
use goblin::elf::reloc::{Rela, R_AARCH64_RELATIVE, R_X86_64_RELATIVE};

impl<'a> Elf<'a> {
    /// Process all relocations for a PIE binary
    fn process_relocations(&self, ttbr0_phys: usize) -> Result<usize, ElfError> {
        if !self.is_pie() {
            return Ok(0);  // ET_EXEC has no runtime relocations
        }
        
        let load_base = self.load_base();
        let mut count = 0;
        
        // Goblin parses .rela.dyn automatically
        for rela in &self.inner.dynrelas {
            self.apply_relocation(ttbr0_phys, load_base, rela)?;
            count += 1;
        }
        
        log::debug!("[ELF] Applied {} relocations", count);
        Ok(count)
    }
}
```

### 4.4 Exit Criteria

- Goblin's `dynrelas` used for relocation iteration
- No manual PT_DYNAMIC parsing needed
- Simpler, less error-prone code

---

## 5. Step 4: Write Architecture-Specific Relocation Handlers

### 5.1 Tasks

1. Use goblin's relocation constants (no manual constants needed)
2. Implement `apply_relocation()` using goblin's `Rela` type
3. Implement `write_user_u64()` helper
4. Call `process_relocations()` from load flow

### 5.2 Code Changes (Following Theseus Pattern)

**File:** `crates/kernel/src/loader/elf.rs`

```rust
use goblin::elf::reloc::*;  // R_AARCH64_RELATIVE, R_X86_64_RELATIVE, etc.

impl<'a> Elf<'a> {
    /// Apply a single relocation using goblin's Rela type
    fn apply_relocation(
        &self,
        ttbr0_phys: usize,
        load_base: usize,
        rela: &goblin::elf::reloc::Rela,
    ) -> Result<(), ElfError> {
        let target_va = load_base + rela.r_offset as usize;
        
        match rela.r_type {
            // aarch64: R_AARCH64_RELATIVE = 1027
            #[cfg(target_arch = "aarch64")]
            R_AARCH64_RELATIVE => {
                let value = (load_base as i64 + rela.r_addend) as u64;
                self.write_user_u64(ttbr0_phys, target_va, value)?;
            }
            
            // x86_64: R_X86_64_RELATIVE = 8
            #[cfg(target_arch = "x86_64")]
            R_X86_64_RELATIVE => {
                let value = (load_base as i64 + rela.r_addend) as u64;
                self.write_user_u64(ttbr0_phys, target_va, value)?;
            }
            
            other => {
                log::trace!("[ELF] Skipping unsupported reloc type {}", other);
            }
        }
        
        Ok(())
    }
    
    /// Write a u64 to user address space
    fn write_user_u64(
        &self,
        ttbr0_phys: usize,
        va: usize,
        value: u64,
    ) -> Result<(), ElfError> {
        let l0_va = mmu::phys_to_virt(ttbr0_phys);
        let l0 = unsafe { &mut *(l0_va as *mut mmu::PageTable) };
        
        let page_va = va & !0xFFF;
        let page_offset = va & 0xFFF;
        
        match mmu::walk_to_entry(l0, page_va, 3, false) {
            Ok(walk) => {
                let phys = walk.table.entry(walk.index).address() + page_offset;
                let ptr = mmu::phys_to_virt(phys) as *mut u64;
                unsafe { ptr.write_unaligned(value); }
                Ok(())
            }
            Err(_) => Err(ElfError::MappingFailed),
        }
    }
}
```

### 5.3 Integration into load()

```rust
pub fn load(&self, ttbr0_phys: usize) -> Result<(usize, usize), ElfError> {
    // ... segment loading ...
    
    // Process relocations for PIE binaries
    if self.is_pie() {
        self.process_relocations(ttbr0_phys)?;
    }
    
    Ok((self.entry_point(), initial_brk))
}
```

### 5.4 Exit Criteria

- R_*_RELATIVE relocations applied correctly
- Goblin constants used (no manual relocation type definitions)
- Unsupported relocations logged and skipped
- No memory corruption

---

## 6. Step 5: Integration Testing

### 6.1 Tasks

1. Build eyra-hello for aarch64
2. Add to initramfs
3. Boot LevitateOS and run binary
4. Verify output
5. Add regression test for ET_EXEC

### 6.2 Test Procedure

```bash
# 1. Build eyra-hello
cd userspace/eyra-hello
cargo build --release --target aarch64-unknown-linux-gnu -Zbuild-std=std,panic_abort

# 2. Copy to initramfs
cp target/aarch64-unknown-linux-gnu/release/eyra-hello ../../initramfs/

# 3. Rebuild kernel with updated initramfs
cd ../..
cargo xtask build --arch aarch64

# 4. Run with VNC
cargo xtask run-vnc --arch aarch64

# 5. In shell, run:
/eyra-hello
```

### 6.3 Expected Output

```
=== Eyra Test on LevitateOS ===

[OK] println! works
[OK] argc = 1
     argv[0] = '/eyra-hello'
[OK] Instant::now() works
[OK] elapsed = <time>
[OK] HashMap works (getrandom ok), value = 42

=== Eyra Test Complete ===
```

### 6.4 Regression Test

Verify existing shell still works:
```bash
# Run existing shell binary (ET_EXEC)
/bin/shell
```

### 6.5 Exit Criteria

- eyra-hello runs and produces expected output
- Existing ET_EXEC binaries still work
- No kernel panics or crashes

---

## 7. UoW Breakdown

If steps are too large for one session, split as follows:

### UoW 3.1: Goblin Integration (Step 1)

**Tasks:** Add goblin dependency, refactor Elf struct to wrap goblin

**Exit:** Goblin compiles in no_std, basic parsing works

### UoW 3.2: Load Base + PIE Detection (Step 2)

**Tasks:** Implement load_base(), adjust segment loading

**Exit:** PIE accepted, segments loaded at offset

### UoW 3.3: Relocation Processing (Steps 3-4)

**Tasks:** Use goblin's dynrelas, implement apply_relocation

**Exit:** Relocations applied, no crashes

### UoW 3.4: Integration (Step 5)

**Tasks:** Build eyra, test on LevitateOS

**Exit:** eyra-hello runs successfully

---

## 8. Rollback Plan

If implementation causes issues:

1. Revert changes to elf.rs
2. Keep ET_DYN rejection
3. Fall back to static linking options from remaining-issues.md

---

## 9. Next Phase

**Phase 4: Integration & Testing** will:
- Run comprehensive tests
- Document any issues found
- Update golden logs if needed

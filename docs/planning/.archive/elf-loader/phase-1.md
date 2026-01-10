# Phase 1: Discovery — ELF Dynamic Loader

**TEAM_352** | ELF Loader Feature  
**Created:** 2026-01-09

---

## 1. Feature Summary

> **Best Practices Reference:** See `docs/planning/elf-loader/best-practices.md` for goblin crate recommendation and Theseus patterns.

### Problem Statement

LevitateOS currently only loads **statically linked ET_EXEC** ELF binaries. Modern toolchains (including Eyra, musl, glibc) produce **PIE (Position-Independent Executable)** binaries with **dynamic relocations**.

When attempting to run such a binary:
1. The ELF loader rejects it (`NotExecutable` error for ET_DYN)
2. Even if loaded, relocations aren't processed
3. Function calls via GOT/PLT crash

### Who Benefits

- **Eyra integration** — Eyra binaries are PIE by default
- **Standard Rust binaries** — `cargo build` produces PIE on Linux
- **Future compatibility** — Most modern binaries are PIE

### Feature Goal

Enable LevitateOS to load and execute dynamically linked PIE binaries by:
1. Supporting ET_DYN ELF type
2. Processing relocations (R_AARCH64_RELATIVE, R_X86_64_RELATIVE)
3. Handling ASLR-style load address randomization (optional)

---

## 2. Success Criteria

| Criterion | Verification |
|-----------|--------------|
| ET_DYN binaries load | ELF loader accepts PIE binaries |
| Relocations processed | R_*_RELATIVE relocations applied |
| Entry point correct | Execution starts at correct address |
| Eyra binary runs | `eyra-hello` prints output on LevitateOS |
| Existing binaries work | Regression test: current ET_EXEC binaries still work |

---

## 3. Current State Analysis

### Current ELF Loader

**Location:** `crates/kernel/src/loader/elf.rs`

**Capabilities:**
- Parses ELF64 headers
- Loads PT_LOAD segments
- Maps pages with correct permissions
- Handles .bss zeroing
- Returns entry point

**Limitations:**
- Only accepts `ET_EXEC` (line 165: `if header.e_type != ET_EXEC`)
- No relocation processing
- No support for PT_DYNAMIC segment
- Load base always 0 (line 284: `pub fn load_base(&self) -> usize { 0 }`)

### What PIE Binaries Need

```
$ readelf -h eyra-hello
Type:                              DYN (Position-Independent Executable file)
Entry point address:               0x23c50  (relative, not absolute!)
```

```
$ readelf -d eyra-hello
Dynamic section at offset 0x4d4a0:
  Tag        Type         Name/Value
 0x0000000000000017 (JMPREL)  0x1d6c8
 0x0000000000000007 (RELA)    0x14490
 ...
```

```
$ readelf -r eyra-hello | head
Relocation section '.rela.dyn' at offset 0x14490 contains 1831 entries:
  Offset          Type           Sym. Value    Sym. Name + Addend
000000000533d8  R_AARCH64_RELATIVE           433d8
000000000533e0  R_AARCH64_RELATIVE           433e0
...
```

### Key Differences: ET_EXEC vs ET_DYN

| Aspect | ET_EXEC | ET_DYN (PIE) |
|--------|---------|--------------|
| Load address | Fixed (e.g., 0x400000) | Relative (can load anywhere) |
| Entry point | Absolute VA | Relative to load base |
| Relocations | None (linker resolved all) | Required at runtime |
| ASLR | Not possible | Possible (optional) |

---

## 4. Codebase Reconnaissance

### Files to Modify

| File | Changes Needed |
|------|----------------|
| `crates/kernel/src/loader/elf.rs` | Accept ET_DYN, process relocations |
| `crates/kernel/src/loader/mod.rs` | May need new relocation module |
| `crates/kernel/src/task/process.rs` | Pass load base to ELF loader |

### Recommended Approach: Use Goblin Crate

Per `docs/planning/stability-maturation/library-audit.md`, the hand-rolled ELF parser should be replaced with **goblin** (HIGH priority).

```toml
goblin = { version = "0.8", default-features = false, features = ["elf64", "alloc"] }
```

**Goblin provides:**
- All relocation constants (`R_AARCH64_RELATIVE`, `R_X86_64_RELATIVE`, etc.)
- Parsed `dynrelas` vector (no manual PT_DYNAMIC parsing needed)
- Battle-tested ELF parsing with edge case handling
- `no_std` compatible with `alloc` feature

This eliminates the need for hand-rolled structs like `Elf64Rela`, `Elf64Dyn`, and constants like `DT_*`.

---

## 5. Constraints

### Must Support

- **aarch64** — Primary architecture
- **x86_64** — Secondary architecture
- **R_*_RELATIVE** — Most common relocation type (95%+ of relocations)

### Nice to Have

- **ASLR** — Randomize load address (security improvement)
- **Symbol resolution** — For R_*_GLOB_DAT (rare in static-pie)

### Out of Scope

- **Shared libraries** — No .so loading (Eyra is static-pie)
- **Lazy binding** — All relocations done at load time
- **TLS relocations** — Thread-local storage (can add later if needed)

---

## 6. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| R_*_RELATIVE not sufficient | Low | High | Log unsupported relocs, add as needed |
| Performance impact | Low | Low | Relocations are O(n), done once |
| Memory corruption bugs | Medium | High | Extensive logging, careful bounds checking |
| Breaks existing binaries | Low | High | Add tests for ET_EXEC regression |

---

## 7. Reference Implementations

### Linux Kernel (binfmt_elf.c)

Linux's ELF loader handles:
- ET_EXEC and ET_DYN
- Load address calculation
- Passes to ld-linux.so for relocations

### musl ld.so

musl's dynamic linker is ~2000 lines and handles:
- Relocation processing
- Symbol resolution
- TLS setup

### Key Algorithm (R_RELATIVE)

```c
// Pseudocode for R_AARCH64_RELATIVE
void apply_relative_reloc(uintptr_t load_base, Elf64_Rela *rela) {
    uintptr_t *target = (uintptr_t *)(load_base + rela->r_offset);
    *target = load_base + rela->r_addend;
}
```

---

## 8. Questions for Phase 2

These questions should be answered during design:

1. **Load address selection**: Fixed (e.g., 0x10000) or randomized (ASLR)?
2. **Unsupported relocations**: Error out or warn and continue?
3. **Architecture abstraction**: Separate reloc handlers per arch or unified?
4. **PT_INTERP handling**: Ignore silently or warn?
5. **Testing strategy**: How to verify relocations are correct?

---

## 9. Next Phase

**Phase 2: Design** will define:
- Relocation processing architecture
- Modified ELF loader flow
- Error handling strategy
- Test plan

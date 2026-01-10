# Phase 2 — Structural Extraction

**TEAM_311**: ABI Stability Refactor
**Parent**: `docs/planning/stability-maturation/`
**Depends On**: Phase 1 complete
**Status**: PARTIALLY COMPLETE (see deferred items below)
**Last Updated**: 2026-01-08

---

## 1. Target Design

### 1.1 New Crate Structure
```
crates/abi/
├── Cargo.toml
└── src/
    ├── lib.rs              # Re-exports all modules
    ├── syscall/
    │   ├── mod.rs          # SyscallNumber trait + arch re-exports
    │   ├── aarch64.rs      # AArch64 syscall numbers (Linux NR)
    │   └── x86_64.rs       # x86_64 syscall numbers (Linux NR)
    ├── errno.rs            # Error codes (Linux-compatible)
    ├── stat.rs             # Stat structure (arch-conditional)
    ├── termios.rs          # Terminal structures
    ├── flags.rs            # O_*, PROT_*, MAP_* constants
    └── dirent.rs           # Directory entry structures
```

### 1.2 Responsibility Mapping

| Old Location | New Location | Notes |
|--------------|--------------|-------|
| `kernel/src/arch/aarch64/mod.rs::SyscallNumber` | `los_abi::syscall::aarch64` | Move enum |
| `kernel/src/arch/x86_64/mod.rs::SyscallNumber` | `los_abi::syscall::x86_64` | Move enum |
| `kernel/src/syscall/mod.rs::errno` | `los_abi::errno` | Consolidate |
| `kernel/src/syscall/mod.rs::errno_file` | DELETE | Duplicate |
| `userspace/libsyscall/src/sysno.rs` | DELETE | Use los_abi |
| `userspace/libsyscall/src/errno.rs` | DELETE | Use los_abi |

### 1.3 Layering
```
┌─────────────────────────────────────────────┐
│                 userspace                    │
│  (init, levbox, libsyscall)                 │
├─────────────────────────────────────────────┤
│              los_abi (NEW)                   │
│  Single source of truth for ABI types       │
├─────────────────────────────────────────────┤
│                  kernel                      │
│  (imports los_abi for dispatch)             │
└─────────────────────────────────────────────┘
```

---

## 2. Extraction Strategy

### 2.1 Order of Extraction
1. **errno.rs** - Simplest, no dependencies
2. **flags.rs** - Constants only
3. **syscall/aarch64.rs** - Copy SyscallNumber enum
4. **syscall/x86_64.rs** - Copy SyscallNumber enum
5. **syscall/mod.rs** - Trait for dispatch
6. **stat.rs** - Arch-conditional, needs cfg
7. **termios.rs** - Arch-conditional
8. **dirent.rs** - Single layout

### 2.2 Coexistence Strategy
**None** - Per user directive: "BREAK THE CODE FIX THE CALLSITES"

All old definitions will be removed immediately after new ones are created. No deprecation period.

---

## 3. Steps

### Step 1 — Create crates/abi Skeleton ✅ COMPLETE (TEAM_311)
- [x] Create `crates/abi/Cargo.toml`
- [x] Create `crates/abi/src/lib.rs`
- [x] Add to workspace `Cargo.toml`

### Step 2 — Extract errno ⏭️ SKIPPED
**Reason**: Userspace already uses `linux-raw-sys` for errno. No need to duplicate.
Kernel errno stays in `kernel/src/syscall/mod.rs` (returns negated values).

### Step 3 — Extract syscall numbers ✅ COMPLETE (TEAM_311)
- [x] Create `crates/abi/src/syscall/mod.rs`
- [x] Create `crates/abi/src/syscall/aarch64.rs`
- [x] Create `crates/abi/src/syscall/x86_64.rs`
- [x] Kernel imports `SyscallNumber` from `los_abi`
- [x] Tests verify values match `linux-raw-sys`
- [ ] Remove Spawn/SpawnArgs → **Phase 3** (need clone+exec first)

### Step 4 — Extract data structures ⏸️ PENDING
- [ ] Create `crates/abi/src/stat.rs`
- [ ] Create `crates/abi/src/termios.rs`
- [ ] Create `crates/abi/src/flags.rs`
- [ ] Add compile-time size assertions

---

## 4. DEFERRED ITEMS (Boot-Critical, Requires Careful Refactoring)

> **IMPORTANT FOR FUTURE TEAMS**: The following items were identified during the library audit
> but deferred because they touch boot-critical code paths. They require careful refactoring
> and extensive testing before deployment.

### Step 5 — Replace ELF Parsing with `goblin` 🔶 DEFERRED
**Status**: Dependency added, refactor deferred
**Risk Level**: HIGH (boot-critical, 520 lines, deep memory management integration)

**What was done**:
- [x] Added `goblin = { version = "0.9", default-features = false, features = ["elf64"] }` to kernel

**What remains**:
- [ ] Replace `kernel/src/loader/elf.rs` hand-rolled parser with goblin
- [ ] Remove ~300 lines of hand-rolled code
- [ ] Verify kernel boots with new parser (both aarch64 and x86_64)

**Why deferred**:
- ELF loader is deeply integrated with kernel memory management (`mm_user::map_user_page`)
- Parsing (goblin can replace) is interleaved with loading (kernel-specific)
- Requires careful extraction of loading logic before goblin integration
- Boot failure risk is high if done incorrectly

**Future team guidance**:
1. First, refactor `Elf::load()` to separate parsing from loading
2. Create a `LoadedElf` struct that holds goblin's parsed ELF
3. Test extensively on both architectures before removing hand-rolled code

### Step 6 — Use `x86_64` crate's GDT/IDT 🔶 DEFERRED
**Status**: Not started
**Risk Level**: CRITICAL (boot fails immediately if wrong)

**What remains**:
- [ ] Replace `crates/hal/src/x86_64/gdt.rs` with `x86_64::structures::gdt`
- [ ] Replace `crates/hal/src/x86_64/idt.rs` with `x86_64::structures::idt`
- [ ] Update all callers (boot, exceptions, task switching)
- [ ] Remove ~200 lines of hand-rolled code

**Why deferred**:
- GDT is loaded during early boot before any error handling is available
- IDT is used for ALL interrupt handling
- Per-CPU structures in `kernel/src/arch/x86_64/cpu.rs` embed `Gdt` and `TaskStateSegment`
- `gdt::set_kernel_stack()` is called during every task switch
- Any mistake = immediate triple fault, no debugging possible

**Future team guidance**:
1. Study how the `x86_64` crate's GDT/IDT differ from hand-rolled
2. Create a test branch and verify boot works
3. Have QEMU debug output enabled during testing
4. Consider incremental migration: IDT first (less critical), then GDT

### Step 7 — Replace Multiboot2 parsing 🔶 DEFERRED (Lower Priority)
**Status**: Not started
**Risk Level**: MEDIUM (x86_64 boot only)

**What remains**:
- [ ] Add `multiboot2 = "0.20"` dependency
- [ ] Replace `crates/hal/src/x86_64/multiboot2.rs` with crate
- [ ] Verify x86_64 boot still works

**Why lower priority**:
- Only affects x86_64 multiboot boot path (not Limine, not aarch64)
- Current implementation works correctly
- Less risky than GDT/IDT but still boot-critical

---

See `phase-2-step-1.md` through `phase-2-step-4.md` for completed step details.
See `library-audit.md` for full analysis of hand-rolled vs crate alternatives.

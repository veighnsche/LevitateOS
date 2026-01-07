# Phase 4: Cleanup — Remove Legacy Boot Code

## Purpose
Delete dead code, remove temporary compatibility layers, and ensure the codebase contains only living, active code per UNIX Rule 6.

---

## UNIX Philosophy: No Dead Code (Rule 6)

> "Remove unused functions, modules, commented-out code, 'kept for reference' logic. Git history exists for this."

After Phase 3, we will have:
- New `boot/` module with BootInfo abstraction
- Old boot.S still present but unused (if Limine is primary)
- Old multiboot parsing code potentially unused
- Duplicate entry points removed

This phase removes everything that's no longer called.

---

## Dead Code Inventory

### x86_64 Boot Code to Remove

| File | Lines | Status After Phase 3 | Action |
|------|-------|---------------------|--------|
| `kernel/src/arch/x86_64/boot.S` | 330 | Unused if Limine primary | DELETE or MINIMIZE |
| `kernel/src/arch/x86_64/linker.ld` multiboot sections | ~20 | Unused | REMOVE sections |
| `crates/hal/src/x86_64/multiboot2.rs` | ~200 | Replaced by boot/multiboot.rs | DELETE |
| Old `kernel_main(magic, info)` | ~80 | Replaced by unified entry | DELETE |

### Potential Savings
- **~630 lines deleted** from x86_64 boot path
- **Cleaner linker script** without multiboot sections
- **Single entry point** instead of two

### What to Keep

| Component | Reason |
|-----------|--------|
| `boot/multiboot.rs` | Legacy QEMU support (optional) |
| Minimal entry stub | If keeping multiboot as fallback |
| AArch64 DTB parsing | Still needed, now in boot/dtb.rs |

---

## Steps

### Step 1: Remove Unused Multiboot Code
Tasks:
1. Delete `crates/hal/src/x86_64/multiboot2.rs` if fully replaced
2. Remove multiboot2 parsing from `los_hal::x86_64`
3. Update HAL exports

**Exit Criteria**: 
- HAL compiles without multiboot2 module
- All tests pass

### Step 2: Minimize or Remove boot.S
Tasks:
1. If Limine is sole path: DELETE `boot.S` entirely
2. If keeping multiboot fallback: Reduce to minimal stub that calls boot/multiboot.rs
3. Remove multiboot1/2 header sections from linker.ld

**Exit Criteria**:
- No 32-bit assembly in kernel (Limine path)
- OR minimal stub only (multiboot fallback path)

### Step 3: Remove Temporary Adapters
Tasks:
1. Delete any `#[deprecated]` wrappers added during migration
2. Remove re-exports that were kept for compatibility
3. Clean up `#[cfg]` attributes that are now always-on

**Exit Criteria**:
- No compatibility shims remain
- Clean, direct API calls only

### Step 4: Tighten Visibility
Tasks:
1. Make `boot/` module internals private
2. Export only `BootInfo` and entry point
3. Hide protocol-specific parsers (multiboot, limine, dtb)

**Exit Criteria**:
- `pub use boot::BootInfo;` is the only export
- Internal modules are `pub(crate)` or private

### Step 5: File Size Audit
Per Rule 7: Files < 500 lines ideal, < 1000 max.

Tasks:
1. Check all new/modified files
2. Split any that exceed limits
3. Ensure logical organization

**Exit Criteria**:
- All files under 1000 lines
- Most under 500 lines

---

## Verification

After cleanup:
```bash
# Ensure nothing broke
cargo build --release
cargo test --workspace --exclude levitate-kernel

# Boot tests
./run-term.sh              # x86_64 via Limine
./run-term.sh --aarch64    # AArch64 via DTB

# Behavior tests
cargo xtask test behavior

# Line count check
find kernel/src/boot -name "*.rs" -exec wc -l {} \;
```

---

## Git Hygiene

When deleting code:
1. **Single commit per logical deletion** - Easy to revert if needed
2. **Commit message references Phase 4** - `TEAM_280: Phase 4 - Remove legacy boot.S`
3. **Don't squash during review** - Keep history clear

---

## TODO Tracking (Rule 11)

Any incomplete work during cleanup must be recorded:

### In Code
```rust
// TODO(TEAM_XXX): Description of what remains
```

### In Global TODO.md
Add items with file, line, and description if work is deferred.

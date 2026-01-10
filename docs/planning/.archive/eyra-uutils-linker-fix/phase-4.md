# Phase 4: Initramfs Integration

**TEAM_368** | Eyra/uutils Integration  
**Created:** 2026-01-10

---

## 1. Phase Summary

**Goal:** Integrate all Eyra-built uutils binaries into the LevitateOS initramfs so they are available at boot time.

**Prerequisite:** Phase 3 complete (TEAM_367 fixed linker issues, all utilities build)

---

## 2. Current State

### What Exists
- 14 Eyra utilities build successfully: cat, pwd, mkdir, ls, echo, env, touch, rm, rmdir, ln, cp, mv, true, false
- `cargo xtask build eyra` command auto-discovers and builds all utilities
- eyra-hello is already copied to initramfs (proof of concept)

### What's Missing
- Other Eyra utilities are NOT copied to initramfs
- `create_initramfs()` in `xtask/src/build/commands.rs` only copies eyra-hello

---

## 3. Implementation Design

### 3.1 Modify `create_initramfs()` 

**File:** `xtask/src/build/commands.rs`

**After** eyra-hello copying (line ~127), add:

```rust
// TEAM_368: Add all Eyra utilities to initramfs
let eyra_utils = [
    "cat", "pwd", "mkdir", "ls", "echo", "env", 
    "touch", "rm", "rmdir", "ln", "cp", "mv", 
    "true", "false"
];
for util in &eyra_utils {
    let src = PathBuf::from(format!(
        "crates/userspace/eyra/{}/target/{}/release/{}", 
        util, eyra_target, util
    ));
    if src.exists() {
        std::fs::copy(&src, root.join(util))?;
        count += 1;
    }
}
```

### 3.2 Build Order Consideration

Eyra utilities must be built BEFORE `create_initramfs()` is called.

**USER DECISION:** Add `--with-eyra` flag to `cargo xtask build all`.

**Implementation:**
```rust
// In BuildCommands::All
All {
    /// Include Eyra-based coreutils in the build
    #[arg(long)]
    with_eyra: bool,
},

// In build_all()
pub fn build_all(arch: &str, with_eyra: bool) -> Result<()> {
    if with_eyra {
        build_eyra(arch, None)?;
    }
    build_userspace(arch)?;
    create_initramfs(arch, with_eyra)?;  // Pass flag
    // ...
}
```

---

## 4. Binary Naming Conflicts

### Potential Issue
Both bare-metal userspace and Eyra provide utilities with same names (cat, ls, etc.).

### Resolution: Replace Bare-Metal Versions Entirely

**USER DECISION:** Eyra versions replace bare-metal versions entirely.

**Rationale:**
- Eyra binaries use battle-tested uutils-coreutils
- More complete feature set
- Consistent behavior with GNU coreutils

**Implementation:**
- Eyra binaries are copied directly to initramfs root (no prefix, no subdirectory)
- If bare-metal version exists with same name, Eyra version overwrites it
- Bare-metal utilities that have no Eyra equivalent remain

---

## 5. Steps

### Step 1: Update create_initramfs() to copy Eyra binaries
- Add Eyra utility list
- Copy from per-utility target directories
- Track count for logging

### Step 2: Handle binary conflicts
- Implement chosen naming strategy
- Update shell or init to find Eyra binaries

### Step 3: Test initramfs creation
- Run `cargo xtask build initramfs`
- Verify Eyra binaries are present in initramfs

### Step 4: Document the change
- Update FUTURE_TEAMS_README.md with completion status

---

## 6. Verification

```bash
# Build Eyra utilities
cargo xtask build eyra

# Build initramfs
cargo xtask build initramfs

# Verify contents
cpio -t < initramfs_x86_64.cpio | grep -E "(cat|ls|echo)"
```

---

## 7. Resolved Questions

### Q1: Binary naming strategy ✅
**Decision:** Replace bare-metal versions entirely (Option C)

### Q2: Automatic build integration ✅
**Decision:** Add `--with-eyra` flag (Option C)

**Usage:**
```bash
cargo xtask build all --with-eyra
```

---

## 8. Dependencies

- **Requires:** Eyra utilities built (`cargo xtask build eyra`)
- **Modifies:** `xtask/src/build/commands.rs`
- **Tests:** Initramfs content verification


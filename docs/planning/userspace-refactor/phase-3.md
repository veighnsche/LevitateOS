# Phase 3 — Migration

**Parent:** Userspace Architecture Refactor  
**File:** `docs/planning/userspace-refactor/phase-3.md`  
**Depends On:** Phase 2 complete

---

## 1. Migration Strategy

### 1.1 Order of Migration

1. **Kernel** — Change from `run_from_initramfs("hello")` to `run_from_initramfs("init")`
2. **Golden File** — Update expected boot output
3. **Initramfs** — Include `init`, `shell` binaries (remove `hello`)
4. **Build Scripts** — Update `make_initramfs.sh` or replace with xtask

### 1.2 Breaking Changes (Rule 5)

| Change | Type | Action |
|--------|------|--------|
| Binary name "hello" → "init" | Breaking | Update kernel + golden file |
| Shell banner moved to init | Observable | Update golden file |
| Old `userspace/hello/` deleted | Cleanup | After migration verified |

---

## 2. Call Site Inventory

### 2.1 Kernel Call Sites

| File | Line | Code | Change |
|------|------|------|--------|
| `kernel/src/main.rs` | 636 | `run_from_initramfs("hello", ...)` | → `"init"` |

### 2.2 Build Scripts

| File | Change |
|------|--------|
| `scripts/make_initramfs.sh` | Add `init`, `shell` binaries to initrd_root |
| `xtask/src/main.rs` | Add `build-userspace` command (optional) |

### 2.3 Documentation

| File | Change |
|------|--------|
| `docs/GOTCHAS.md` | Update userspace build instructions |
| `docs/ROADMAP.md` | Add Phase 8c or document refactor |
| `tests/golden_boot.txt` | Update expected output |

---

## 3. Phase 3 Steps

### Step 1: Update Kernel to Run "init"
```diff
- task::process::run_from_initramfs("hello", &archive);
+ task::process::run_from_initramfs("init", &archive);
```

### Step 2: Update Golden File
Expected new output:
```
[INIT] LevitateOS init v0.1
[INIT] Spawning shell...

LevitateOS Shell (lsh) v0.1
Type 'help' for available commands.

# 
```

### Step 3: Update initramfs Build
**Option A:** Update `scripts/make_initramfs.sh`
```bash
#!/bin/bash
# Build userspace
cd userspace
cargo build --release
cd ..

# Copy binaries
mkdir -p initrd_root
cp userspace/target/aarch64-unknown-none/release/init initrd_root/
cp userspace/target/aarch64-unknown-none/release/shell initrd_root/

# Create CPIO
cd initrd_root
find . | cpio -o -H newc > ../initramfs.cpio
cd ..
```

**Option B:** Add `cargo xtask build-userspace` (cleaner)
- Build userspace workspace
- Copy binaries to initrd_root
- Generate initramfs.cpio
- Touch kernel to trigger rebuild

### Step 4: Verify All Tests Pass
- `cargo xtask test all`
- `cargo xtask run-vnc` (visual verification)

---

## 4. Rollback Plan

If migration causes issues:
1. Revert kernel to `run_from_initramfs("hello")`
2. Keep old `userspace/hello/` until stable
3. Golden file can be reverted with git

---

## 5. Exit Criteria

- [ ] Kernel runs `init` (not `hello`)
- [ ] `init` spawns/execs `shell`
- [ ] Shell behavior unchanged (same prompt, same commands)
- [ ] `cargo xtask test all` passes
- [ ] VNC visual shows working shell
- [ ] Golden file updated and passing

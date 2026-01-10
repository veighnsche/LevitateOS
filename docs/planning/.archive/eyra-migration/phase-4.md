# Phase 4: Cleanup

**TEAM_362** | Refactor Userspace to Eyra/std  
**Created:** 2026-01-09

---

## 1. Dead Code Removal (Rule 6)

### 1.1 Directories to Delete

| Directory | Status | Action |
|-----------|--------|--------|
| `crates/userspace/ulib/` | ✅ DELETED | Already removed |
| `crates/userspace/init/` | Pending | Delete after migration |
| `crates/userspace/shell/` | Pending | Delete after migration |
| `crates/userspace/levbox/` | Pending | Delete after migration |
| `crates/userspace/repro_crash/` | Pending | Delete (debug-only) |
| `crates/userspace/systest/` | Pending | Delete (superseded by Eyra tests) |

### 1.2 Test Binaries Decision

levbox contains 9 test binaries that need a decision:

| Binary | Decision | Rationale |
|--------|----------|----------|
| `suite_test_core` | Delete | Superseded by golden tests |
| `clone_test` | Keep as Eyra test | Useful for syscall testing |
| `mmap_test` | Keep as Eyra test | Useful for syscall testing |
| `pipe_test` | Keep as Eyra test | Useful for syscall testing |
| `signal_test` | Keep as Eyra test | Useful for syscall testing |
| `tty_test` | Keep as Eyra test | Useful for TTY testing |
| `pty_test` | Keep as Eyra test | Useful for PTY testing |
| `pty_interact` | Delete | Interactive debug tool |
| `interrupt_test` | Delete | Ctrl+C simulation, debug only |
| `test_runner` | Delete | Superseded by xtask test |

**Action:** Migrate useful tests to `crates/userspace/eyra/tests/` with Eyra, delete others.

### 1.3 Files to Update

| File | Changes |
|------|---------|
| `crates/userspace/Cargo.toml` | Remove old members, add new |
| `scripts/make_initramfs.sh` | Update binary paths |
| `xtask/src/build/` | Update userspace build |

---

## 2. Temporary Adapter Removal

No temporary adapters expected — clean cutover from old to new.

---

## 3. Encapsulation Tightening

### 3.1 libsyscall Review

After migration, review libsyscall:
- Remove any functions only used by ulib
- Keep only syscalls needed by Eyra apps
- Document which syscalls are LevitateOS-specific

### 3.2 Exported Symbols

Ensure each Eyra app only exports `main` (handled automatically by std).

---

## 4. Documentation Updates

### 4.1 Files to Update

| File | Changes |
|------|---------|
| `docs/ARCHITECTURE.md` | Update userspace section |
| `crates/userspace/README.md` | Document Eyra build |
| `README.md` | Update build instructions |

### 4.2 New Documentation

Create `docs/USERSPACE.md`:
- How to build userspace apps
- Eyra dependency setup
- Adding new apps
- LevitateOS-specific syscalls

---

## 5. Phase 4 Steps

### Step 1: Delete Old Directories
```bash
rm -rf crates/userspace/init
rm -rf crates/userspace/shell
rm -rf crates/userspace/levbox
rm -rf crates/userspace/repro_crash
rm -rf crates/userspace/systest
```

### Step 2: Update Workspace Cargo.toml
- Point to new `apps/` structure
- Remove old members

### Step 3: Update Build Scripts
- `scripts/make_initramfs.sh`
- `xtask/src/build/`

### Step 4: Update Documentation
- Architecture docs
- README files
- Build instructions

### Step 5: Final Verification
- Full clean build
- All tests pass
- Boot and test all commands

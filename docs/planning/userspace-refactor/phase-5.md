# Phase 5 — Hardening and Handoff

**Parent:** Userspace Architecture Refactor  
**File:** `docs/planning/userspace-refactor/phase-5.md`  
**Depends On:** Phase 4 complete

---

## 1. Final Verification

### 1.1 Test Suite
- [ ] `cargo xtask test all` passes
- [ ] `cargo xtask test behavior` — golden file matches
- [ ] `cargo xtask test regression` — 27+ checks pass

### 1.2 Visual Verification
- [ ] `cargo xtask run-vnc` — shell visible on GPU
- [ ] `echo hello` works
- [ ] `clear` works
- [ ] `exit` works (should show exit message)

### 1.3 Build Verification
- [ ] `cargo xtask build-userspace` (if implemented) works
- [ ] Or: `cd userspace && cargo build --release` works
- [ ] Initramfs includes `init`, `shell`

---

## 2. Documentation Updates

### 2.1 Code Comments

| File | Update |
|------|--------|
| `libsyscall/src/lib.rs` | Module doc: "Userspace syscall library for LevitateOS" |
| `init/src/main.rs` | "PID 1 init process" |
| `shell/src/main.rs` | "Interactive shell" |

### 2.2 Project Documentation

| File | Update |
|------|--------|
| `docs/ROADMAP.md` | Add Phase 8c (spawn syscall) or mark userspace refactor complete |
| `docs/GOTCHAS.md` | Update userspace build instructions |
| `userspace/README.md` | NEW: Document userspace structure |
| `docs/testing/behavior-inventory.md` | Add init process behaviors |

### 2.3 README for Userspace

**File:** `userspace/README.md`
```markdown
# LevitateOS Userspace

This directory contains userspace applications for LevitateOS.

## Structure

- `libsyscall/` — Shared syscall library (no_std)
- `init/` — PID 1 init process
- `shell/` — Interactive shell (lsh)

## Building

```bash
cd userspace
cargo build --release
```

Binaries are output to `target/aarch64-unknown-none/release/`.

## Adding to Initramfs

Copy binaries to `initrd_root/` and run:
```bash
./scripts/make_initramfs.sh
```

## Syscall ABI

See `libsyscall/src/lib.rs` for syscall numbers and wrappers.
```

---

## 3. Handoff Notes

### 3.1 Remaining Work

| Item | Priority | Notes |
|------|----------|-------|
| Spawn syscall | Medium | Enables init to spawn shell as child process |
| Heap allocator for userspace | Low | Currently all userspace is heap-free |
| Coreutils expansion | Low | Add `cat`, `ls`, etc. |

### 3.2 Known Limitations

1. **No spawn syscall** — init currently execs into shell (becomes shell), cannot respawn on exit
2. **No heap** — userspace apps cannot allocate dynamic memory
3. **No filesystem access** — only stdin/stdout/stderr work

### 3.3 Future Improvements

- Implement `spawn` syscall (Phase 8c)
- Add `open`, `close`, `read` for file access
- Add userspace heap via `sbrk` implementation

---

## 4. Handoff Checklist

- [ ] Project builds cleanly (`cargo build --release`)
- [ ] All tests pass (`cargo xtask test all`)
- [ ] Behavioral regression tests pass
- [ ] Team file updated with completion status
- [ ] ROADMAP.md updated
- [ ] GOTCHAS.md updated
- [ ] userspace/README.md created
- [ ] behavior-inventory.md updated with init behaviors
- [ ] Any remaining TODOs tagged with `TODO(TEAM_118)`
- [ ] No open questions blocking handoff

---

## 5. Behavior Inventory Additions

Add to `docs/testing/behavior-inventory.md`:

### Group 13: Init Process

| ID | Behavior | Tested |
|----|----------|--------|
| INIT-1 | Init prints startup banner | [ ] |
| INIT-2 | Init spawns/execs shell | [ ] |
| INIT-3 | Shell prompt appears after init | [ ] |

---

## 6. Success Metrics

| Metric | Before | After |
|--------|--------|-------|
| Syscall duplication | 2 copies (~60 lines each) | 1 copy (libsyscall) |
| Userspace apps | 2 (hello, shell — duplicates) | 3 (libsyscall, init, shell) |
| Build commands | Manual per-app | `cargo build` in workspace |
| Init process | None (kernel runs "hello") | Proper PID 1 |

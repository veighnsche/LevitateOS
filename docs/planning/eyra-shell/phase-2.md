# Phase 2: Design

## Objective

Define the architecture for brush shell running on Eyra/LevitateOS.

## Design Decisions (ANSWERED)

### Q1: POSIX/Bash Compatibility
**Question:** Do we need bash-compatible syntax?

**Answer:** ✅ **YES** — brush provides full POSIX and Bash compatibility out of the box.

---

### Q2: Syscall Backend
**Question:** How do we adapt brush's syscall layer?

**Answer:** ✅ **Use Eyra's std directly** — NO SHIMS.

brush uses the `nix` crate for POSIX APIs. Eyra provides Linux-compatible syscalls, so this should work directly.

---

### Q3: Line Editor
**Question:** Which line editor to use?

**Answer:** ✅ **reedline** — brush already uses it!

---

### Q4: Build Integration
**Question:** How to integrate into LevitateOS build?

**Answer:** ✅ **Add to Eyra workspace** — Like other coreutils.

---

### Q5: Init Integration
**Question:** How does init spawn the new shell?

**Answer:** Spawn "brush" binary (or symlink "shell" → "brush").

---

## Proposed Architecture

```
crates/userspace/eyra/
├── brush/                  ← brush Shell (Eyra-adapted)
│   ├── Cargo.toml
│   ├── build.rs           ← -nostartfiles for Eyra
│   └── src/
│       └── main.rs        ← Wrapper around brush-shell crate
```

Or vendor the brush crates if modifications needed.

## Key Integration Points

1. **Process spawning** — brush uses fork/exec, Eyra must support this
2. **Signals** — SIGINT, SIGCHLD, etc. for job control
3. **TTY/Terminal** — For interactive line editing
4. **File I/O** — For script execution
5. **Environment variables** — PATH, HOME, etc.

## Success Criteria

- [x] All design questions answered
- [x] Architecture documented
- [ ] Syscall gap analysis from Phase 1
- [ ] No open blockers for implementation

# Phase 1 Step 1 — Inventory Current State

**TEAM_311**: ABI Stability Refactor
**Parent**: `phase-1.md`
**Status**: Ready for Execution

---

## Objective
Inventory all current syscall usages, spawn callsites, and verify baseline tests pass.

---

## Tasks

### 1.1 List All Syscall Usages in Userspace
```bash
grep -rn "syscall[0-9]" userspace/libsyscall/src/
grep -rn "__NR_\|SYS_" userspace/
```

### 1.2 List All spawn/spawn_args Callsites
```bash
grep -rn "spawn\|spawn_args" userspace/
grep -rn "sys_spawn" kernel/
```

### 1.3 Verify Golden Tests Pass
```bash
cargo xtask test --arch aarch64
cargo xtask test --arch x86_64
```

### 1.4 Verify Library Tests Pass
```bash
cargo test -p los_utils -p los_error --features std
```

---

## Expected Outputs

1. **Syscall Usage List**: File listing all syscall invocations in userspace
2. **Spawn Callsite List**: All locations using spawn/spawn_args
3. **Test Results**: Confirmation all baseline tests pass

---

## Exit Criteria
- [ ] Syscall inventory documented
- [ ] Spawn callsites documented  
- [ ] All tests pass
- [ ] Ready to proceed to Step 2

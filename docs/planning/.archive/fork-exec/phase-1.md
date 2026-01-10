# Phase 1 — Discovery

**TEAM_312**: Kernel Fork/Exec Implementation
**Parent**: `docs/planning/fork-exec/`
**Status**: In Progress
**Created**: 2026-01-08

---

## 1. Feature Summary

### Problem Statement
The kernel uses custom syscalls (`Spawn`, `SpawnArgs` - numbers 1000, 1001) for process 
creation instead of the Linux-standard `fork()` + `execve()` pattern. This prevents:
- Linux binary compatibility
- Standard POSIX process creation patterns
- Removal of deprecated custom syscalls from `los_abi`

### Who Benefits
- **Userspace programs**: Can use standard fork+exec patterns
- **Shell**: Standard job control with fork
- **Future**: Linux binary compatibility

### Success Criteria
1. `fork()` creates a child process with copied address space
2. `execve()` replaces current process with new ELF image
3. All existing `spawn()` callsites migrated to `fork()`+`exec()`
4. `Spawn`/`SpawnArgs` syscalls removed from kernel and `los_abi`
5. All tests pass

---

## 2. Current State Analysis

### 2.1 Process Creation Today

**Via Spawn syscall (custom):**
```
User calls spawn("/bin/ls")
  → syscall(1000, path_ptr, path_len)
    → sys_spawn() in kernel
      → Find ELF in initramfs
      → create_user_page_table() - new address space
      → elf.load(ttbr0_phys) - load segments
      → setup_user_stack()
      → Create UserTask with new PID
      → Register in process_table
      → Add to scheduler
    → Return child PID to parent
```

**Via Clone syscall (threads only):**
```
User calls clone(CLONE_VM | CLONE_THREAD, stack, ...)
  → syscall(220/56, flags, stack, ...)
    → sys_clone() in kernel
      → IF !CLONE_VM || !CLONE_THREAD: return ENOSYS  ← BLOCKER
      → Share parent's page table (ttbr0)
      → Create new TCB with new TID
      → Clone parent's registers, set SP/TLS
      → Add to scheduler
    → Return child TID to parent, 0 to child
```

**Via Exec syscall (stub):**
```
User calls exec("/bin/ls")
  → syscall(221/59, path_ptr, path_len)
    → sys_exec() in kernel
      → log::warn!("exec is currently a stub")
      → return ENOSYS  ← BLOCKER
```

### 2.2 What's Missing

| Capability | Current | Needed |
|------------|---------|--------|
| Fork (copy address space) | ENOSYS | Full implementation |
| Exec (replace process image) | ENOSYS | Full implementation |
| vfork (suspend parent) | N/A | Optional (optimization) |
| COW (copy-on-write) | N/A | Desirable but not required |

---

## 3. Codebase Reconnaissance

### 3.1 Key Files to Modify

| File | Current Role | Changes Needed |
|------|--------------|----------------|
| `kernel/src/syscall/process.rs` | sys_clone (threads), sys_spawn, sys_exec | Implement fork-clone, fix exec |
| `kernel/src/task/process.rs` | spawn_from_elf() | May need exec_into() variant |
| `kernel/src/task/thread.rs` | create_thread() | create_process() for fork |
| `kernel/src/memory/user.rs` | create_user_page_table() | copy_user_page_table() for fork |
| `kernel/src/loader/elf.rs` | Elf::load() | exec_replace() variant |

### 3.2 Key Functions

**For Fork:**
- `create_user_page_table()` → Need `copy_user_page_table(parent_ttbr0)`
- `sys_clone()` → Handle fork case (no CLONE_VM, no CLONE_THREAD)

**For Exec:**
- `spawn_from_elf_with_args()` → Need variant that replaces current process
- `sys_exec()` → Full implementation

### 3.3 Memory Model

Current model:
- Each process has its own `ttbr0` (page table root)
- Threads share parent's `ttbr0` (CLONE_VM)
- No copy-on-write - pages are copied immediately

Fork needs:
- Copy parent's user page tables to new tables
- Copy all mapped user pages (or implement COW)
- Child gets copy of parent's heap, stack, data

### 3.4 Affected Callsites (After Implementation)

| File | Current | After |
|------|---------|-------|
| `userspace/init/src/main.rs` | `spawn("shell")` | `fork()` then child `exec("shell")` |
| `userspace/levbox/test_runner.rs` | `spawn(test)` | `fork()` + `exec()` |
| `userspace/levbox/suite_test_core.rs` | `spawn_args()` | `fork()` + `exec_args()` |
| `userspace/shell/src/main.rs` | `spawn_args()` | `fork()` + `exec_args()` |

---

## 4. Constraints

### 4.1 Performance
- Fork without COW copies all pages → acceptable for now
- Future optimization: implement COW

### 4.2 Compatibility  
- Must follow Linux syscall semantics
- `fork()` returns 0 to child, PID to parent
- `execve()` only returns on error

### 4.3 Architecture Support
- Must work on both aarch64 and x86_64
- Use architecture-independent abstractions where possible

### 4.4 Memory Safety
- Child must not share parent's heap allocations
- Child must have independent kernel stack

---

## 5. Open Questions (For Phase 2)

### Q1: COW or Eager Copy?
Options:
- **Eager copy**: Copy all pages immediately (simpler, slower)
- **COW**: Mark pages read-only, copy on write (complex, faster)

Recommendation: Start with eager copy, COW is optimization.

### Q2: What happens to open file descriptors?
Options:
- Fork: Copy FD table (standard Unix behavior)
- Exec: Keep FD table unless O_CLOEXEC

### Q3: What happens to pending signals?
Need to define signal inheritance behavior.

### Q4: Should we implement vfork?
vfork suspends parent until child execs - avoids copying address space.
Recommendation: Not for initial implementation.

---

## 6. Steps

### Step 1 — Analyze spawn_from_elf Flow ✅
- [x] Read `kernel/src/task/process.rs`
- [x] Understand page table creation
- [x] Understand ELF loading

### Step 2 — Analyze clone/thread Flow ✅
- [x] Read `kernel/src/syscall/process.rs` sys_clone
- [x] Read `kernel/src/task/thread.rs`
- [x] Understand what's shared vs copied

### Step 3 — Analyze Memory Model ✅
- [x] Read `kernel/src/memory/user.rs`
- [x] Understand page table structure
- [x] Identify what needs copying for fork

### Step 4 — Document API Contract ✅
- [x] Define fork() behavior precisely → See Phase 2 Section 2
- [x] Define execve() behavior precisely → See Phase 2 Section 3
- [x] Move to Phase 2

---

## Next: Phase 2 — Design

Phase 2 will define:
1. Exact fork behavior (what's copied, what's shared)
2. Exact exec behavior (what's replaced, what's kept)
3. Implementation approach
4. Answer open questions

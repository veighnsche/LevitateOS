# Phase 2: Root Cause Analysis

**TEAM_339** | Linux ABI Compatibility Bugfix

## Hypotheses

### H1: Deliberate Design Decision (CONFIRMED)
The length-counted string pattern `(ptr, len)` was intentionally chosen for safety reasons. This is a design decision, not a bug.

**Evidence:** Consistent pattern across all path syscalls, no null-termination scanning.

**Confidence:** High

### H2: Incremental Development
Syscalls were added one-by-one without a Linux ABI spec to follow.

**Evidence:** Different teams implemented syscalls at different times.

**Confidence:** Medium

## Key Code Areas

### 1. Syscall Dispatcher
`crates/kernel/src/syscall/mod.rs:35-228`

The dispatcher extracts arguments from `SyscallFrame` and routes to handlers.

### 2. Path-Handling Syscalls
All use the pattern:
```rust
pub fn sys_openat(path: usize, path_len: usize, flags: u32) -> i64
```
Instead of Linux:
```c
int openat(int dirfd, const char *pathname, int flags, mode_t mode)
```

### 3. Userspace Wrappers
`crates/userspace/libsyscall/src/fs.rs`

Matches kernel's non-Linux signatures.

## Investigation Strategy

### Category A: Quick Fixes (1-2 UoW each)
- `__NR_pause` architecture fix
- Duplicate errno cleanup
- Magic number replacement

### Category B: Signature Changes (2-3 UoW each)
Each path syscall needs:
1. Kernel signature change
2. Userspace wrapper change
3. Test update

### Category C: Struct Alignment (1-2 UoW each)
- Stat struct verification
- Termios struct verification

---

## Steps

### Step 1: Classify All Discrepancies

**Tasks:**
1. Assign each discrepancy to Category A, B, or C
2. Estimate UoW count for each
3. Identify dependencies between fixes

**Output:** Classified discrepancy list with effort estimates

### Step 2: Map Change Dependencies

**Tasks:**
1. For each syscall, identify kernel → userspace dependency
2. Order changes to avoid breaking builds
3. Identify test coverage gaps

**Output:** Dependency graph and change order

### Step 3: Define Change Batches

**Tasks:**
1. Group related syscalls for atomic changes
2. Define "checkpoint" tests between batches
3. Plan rollback points

**Output:** Batched change plan for Phase 4

---

## Exit Criteria

- [ ] All discrepancies classified
- [ ] Change dependencies mapped
- [ ] Batched change plan ready
- [ ] Ready for Phase 3 fix design

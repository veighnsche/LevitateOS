# Phase 2 — Design

**TEAM_312**: Kernel Fork/Exec Implementation
**Parent**: `docs/planning/fork-exec/`
**Depends On**: Phase 1 complete
**Status**: In Progress
**Created**: 2026-01-08

---

## 1. Proposed Solution

### 1.1 High-Level Design

**Fork Implementation:**
```
sys_clone(flags=SIGCHLD, stack=0, ...)
  → Detect fork case: !(CLONE_VM | CLONE_THREAD)
  → Copy parent's address space (eager copy, no COW initially)
  → Create new TCB with new PID
  → Clone parent's registers, set return value
  → Parent returns child PID, child returns 0
```

**Exec Implementation:**
```
sys_exec(path_ptr, path_len)  
  → Validate path, find ELF in initramfs
  → Tear down current user address space (keep page table root)
  → Load new ELF into address space
  → Reset stack with new argv/envp
  → Jump to new entry point (never returns)
```

### 1.2 Key Insight

The existing `spawn_from_elf()` already does most of what we need:
- Creates new page table
- Loads ELF
- Sets up stack
- Creates UserTask

For **fork**: We need `copy_address_space()` instead of creating empty tables.
For **exec**: We need to reuse current process but replace its memory contents.

---

## 2. Fork Design

### 2.1 Fork Behavior Contract

| Aspect | Parent | Child |
|--------|--------|-------|
| Return value | Child PID (> 0) | 0 |
| Address space | Original | **Copy** (independent) |
| Page tables | Original | **New** (copied from parent) |
| Registers | Unchanged | Cloned from parent |
| PID | Unchanged | **New** |
| PPID | Unchanged | Parent's PID |
| FD table | Unchanged | **Cloned** (shared underlying files) |
| Pending signals | Unchanged | **Cleared** |
| Kernel stack | Original | **New** |

### 2.2 Implementation Approach

**New function: `copy_user_address_space(parent_ttbr0) -> Option<usize>`**

```rust
// kernel/src/memory/user.rs
pub fn copy_user_address_space(parent_ttbr0: usize) -> Option<usize> {
    // 1. Create new L0 table
    let child_ttbr0 = create_user_page_table()?;
    
    // 2. Walk parent's page tables
    // 3. For each mapped user page:
    //    a. Allocate new physical page
    //    b. Copy contents from parent's page
    //    c. Map in child's page table with same flags
    
    // 4. Return child's page table root
    Some(child_ttbr0)
}
```

**Modify: `sys_clone()` in `kernel/src/syscall/process.rs`**

```rust
pub fn sys_clone(flags, stack, parent_tid, tls, child_tid, tf) -> i64 {
    let is_thread = (flags & CLONE_VM != 0) && (flags & CLONE_THREAD != 0);
    
    if is_thread {
        // Existing thread code path
        return create_thread_clone(...);
    }
    
    // Fork case: create new process with copied address space
    let parent = current_task();
    
    // Copy address space
    let child_ttbr0 = copy_user_address_space(parent.ttbr0)?;
    
    // Create new TCB
    let child = create_forked_process(parent, child_ttbr0, tf)?;
    
    // Clone FD table
    let child_fds = parent.fd_table.lock().clone();
    
    // Register in process table
    process_table::register_process(child.id.0, parent.id.0, child.clone());
    
    // Add to scheduler
    scheduler::add_task(child);
    
    // Return child PID to parent
    child.id.0 as i64
}
```

### 2.3 Page Walking Algorithm

```rust
fn copy_user_pages(parent_l0_phys: usize, child_l0_phys: usize) -> Result<(), MmuError> {
    let parent_l0 = phys_to_virt(parent_l0_phys) as *const PageTable;
    let child_l0 = phys_to_virt(child_l0_phys) as *mut PageTable;
    
    // Walk L0 entries (indices 0-255 for user space, 256-511 for kernel)
    for l0_idx in 0..256 {  // Only user space
        let l0_entry = (*parent_l0).entry(l0_idx);
        if !l0_entry.is_valid() { continue; }
        
        // Recurse into L1, L2, L3...
        copy_table_recursive(l0_entry, child_l0, l0_idx, 1)?;
    }
    Ok(())
}
```

---

## 3. Exec Design

### 3.1 Exec Behavior Contract

| Aspect | Before Exec | After Exec |
|--------|-------------|------------|
| Process image | Old ELF | **New ELF** |
| Address space | Old mappings | **New mappings** |
| PID | P | P (unchanged) |
| PPID | X | X (unchanged) |
| FD table | Original | **Kept** (close O_CLOEXEC) |
| Registers | Old state | **Reset** to entry point |
| Stack | Old contents | **New** argv/envp |
| Return | Only on error | Only on error |

### 3.2 Implementation Approach

**Modify: `sys_exec()` in `kernel/src/syscall/process.rs`**

```rust
pub fn sys_exec(path_ptr: usize, path_len: usize) -> i64 {
    let task = current_task();
    
    // 1. Validate and read path
    let path = copy_user_string(task.ttbr0, path_ptr, path_len)?;
    
    // 2. Find ELF in initramfs
    let elf_data = find_elf_in_initramfs(&path)?;
    
    // 3. Clear current user address space (keep kernel mappings)
    clear_user_address_space(task.ttbr0);
    
    // 4. Parse and load new ELF
    let elf = Elf::parse(&elf_data)?;
    let (entry_point, brk) = elf.load(task.ttbr0)?;
    
    // 5. Set up new stack
    let stack_top = setup_user_stack(task.ttbr0, STACK_PAGES)?;
    
    // 6. Update task state
    task.set_entry_point(entry_point);
    task.set_stack_pointer(stack_top);
    task.set_brk(brk);
    
    // 7. Close O_CLOEXEC file descriptors
    task.fd_table.lock().close_cloexec();
    
    // 8. Jump to new entry point (never returns)
    jump_to_user(entry_point, stack_top);
}
```

**New function: `clear_user_address_space(ttbr0)`**

```rust
pub fn clear_user_address_space(ttbr0_phys: usize) {
    // Walk user page tables (indices 0-255)
    // Free all user physical pages
    // Clear page table entries (but keep the table structure)
}
```

---

## 4. Behavioral Decisions

### 4.1 Fork Edge Cases

| Scenario | Behavior |
|----------|----------|
| Fork with no memory | Return -ENOMEM |
| Fork in signal handler | Allowed (standard Unix) |
| Fork with threads | Only calling thread is forked |
| Child modifies memory | Parent unaffected (separate copy) |

### 4.2 Exec Edge Cases

| Scenario | Behavior |
|----------|----------|
| Exec non-existent file | Return -ENOENT |
| Exec non-ELF file | Return -ENOEXEC |
| Exec with bad permissions | Return -EACCES (future) |
| Exec replaces memory | All old mappings freed |

### 4.3 Answer to Open Questions

**Q1: COW or Eager Copy?**
→ **Eager copy** for initial implementation. COW is a future optimization.

**Q2: What happens to open file descriptors?**
→ **Fork**: Clone FD table (standard Unix)
→ **Exec**: Keep FDs, close those with O_CLOEXEC

**Q3: What happens to pending signals?**
→ **Fork**: Child starts with no pending signals
→ **Exec**: Pending signals preserved (standard)

**Q4: Should we implement vfork?**
→ **No** for initial implementation. vfork is optimization.

---

## 5. API Summary

### 5.1 New Kernel Functions

| Function | Location | Purpose |
|----------|----------|---------|
| `copy_user_address_space()` | `memory/user.rs` | Copy parent's pages for fork |
| `clear_user_address_space()` | `memory/user.rs` | Free user pages for exec |
| `create_forked_process()` | `task/process.rs` | Create child TCB from parent |

### 5.2 Modified Kernel Functions

| Function | Location | Change |
|----------|----------|--------|
| `sys_clone()` | `syscall/process.rs` | Add fork case |
| `sys_exec()` | `syscall/process.rs` | Full implementation |

### 5.3 Userspace API

```rust
// libsyscall/src/process.rs

/// Fork the current process.
/// Returns: 0 to child, child PID to parent, negative errno on error.
pub fn fork() -> isize {
    clone(SIGCHLD as u64, 0, null_mut(), 0, null_mut())
}

/// Replace current process with new executable.
/// Only returns on error.
pub fn exec(path: &str) -> isize {
    syscall2(__NR_execve, path.as_ptr() as u64, path.len() as u64) as isize
}
```

---

## 6. Implementation Order

### Step 1: Memory primitives
1. `copy_user_address_space()` - copy page tables and pages
2. `clear_user_address_space()` - free user pages

### Step 2: Fork implementation  
1. Modify `sys_clone()` to detect fork case
2. Create `create_forked_process()` 
3. Test fork standalone

### Step 3: Exec implementation
1. Implement full `sys_exec()`
2. Test exec standalone

### Step 4: Integration
1. Add `fork()` to libsyscall
2. Test fork+exec pattern
3. Migrate spawn callsites

### Step 5: Cleanup
1. Remove Spawn/SpawnArgs from kernel
2. Remove from los_abi
3. Final testing

---

## 7. Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Memory leaks in fork | Careful page tracking, test with allocator stats |
| Exec leaves orphan pages | clear_user_address_space() must be thorough |
| Fork performance (eager copy) | Accept for now, COW is future work |
| Architecture differences | Use existing MMU abstractions |

---

## Next: Phase 3 — Implementation

Phase 3 will provide step-by-step implementation files.

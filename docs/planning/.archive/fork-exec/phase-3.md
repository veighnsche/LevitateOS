# Phase 3 — Implementation

**TEAM_312**: Kernel Fork/Exec Implementation
**Parent**: `docs/planning/fork-exec/`
**Depends On**: Phase 2 complete
**Status**: Ready for Implementation
**Created**: 2026-01-08

---

## Implementation Order

```
Step 1: Memory primitives (copy/clear address space)
Step 2: Fork implementation (sys_clone fork case)
Step 3: Exec implementation (sys_exec full)
Step 4: Userspace API (fork() wrapper)
Step 5: Integration (migrate spawn callsites)
```

---

## Step 1 — Memory Primitives

**File**: `kernel/src/memory/user.rs`

### 1.1 Add `copy_user_address_space()`

```rust
/// TEAM_312: Copy a user address space for fork.
/// 
/// Creates a new page table and copies all user pages from parent.
/// Uses eager copy (no COW).
///
/// # Arguments
/// * `parent_ttbr0` - Physical address of parent's L0 page table
///
/// # Returns
/// Physical address of child's L0 page table, or None on allocation failure.
pub fn copy_user_address_space(parent_ttbr0: usize) -> Option<usize> {
    // 1. Create new L0 table for child
    let child_ttbr0 = create_user_page_table()?;
    
    // 2. Copy all user pages
    if let Err(_) = copy_user_pages(parent_ttbr0, child_ttbr0) {
        // Cleanup on failure
        free_user_address_space(child_ttbr0);
        return None;
    }
    
    Some(child_ttbr0)
}

fn copy_user_pages(parent_ttbr0: usize, child_ttbr0: usize) -> Result<(), MmuError> {
    // Walk parent's L0 table (user space entries only: 0-255 for aarch64, 0-255 for x86_64)
    let parent_l0 = unsafe { &*(mmu::phys_to_virt(parent_ttbr0) as *const PageTable) };
    
    for l0_idx in 0..256 {
        let l0_entry = parent_l0.entry(l0_idx);
        if !l0_entry.is_valid() { continue; }
        
        // Recurse into L1
        copy_table_level(parent_ttbr0, child_ttbr0, l0_idx, 0)?;
    }
    Ok(())
}
```

### 1.2 Add `clear_user_address_space()`

```rust
/// TEAM_312: Clear user address space for exec.
///
/// Frees all user pages and clears page table entries.
/// Keeps the L0 table and kernel mappings intact.
///
/// # Arguments
/// * `ttbr0_phys` - Physical address of L0 page table
pub fn clear_user_address_space(ttbr0_phys: usize) {
    let l0 = unsafe { &mut *(mmu::phys_to_virt(ttbr0_phys) as *mut PageTable) };
    
    // Walk and free user entries (0-255)
    for l0_idx in 0..256 {
        let l0_entry = l0.entry(l0_idx);
        if !l0_entry.is_valid() { continue; }
        
        // Recurse and free pages
        free_table_recursive(l0_entry, 0);
        
        // Clear the L0 entry
        l0.clear_entry(l0_idx);
    }
    
    // Flush TLB
    mmu::tlb_flush_all();
}
```

### 1.3 Exit Criteria
- [ ] `copy_user_address_space()` compiles
- [ ] `clear_user_address_space()` compiles
- [ ] Kernel builds

---

## Step 2 — Fork Implementation

**File**: `kernel/src/syscall/process.rs`

### 2.1 Modify `sys_clone()` for fork case

```rust
pub fn sys_clone(
    flags: u64,
    stack: usize,
    parent_tid: usize,
    tls: usize,
    child_tid: usize,
    tf: &crate::arch::SyscallFrame,
) -> i64 {
    let is_thread = (flags & CLONE_VM != 0) && (flags & CLONE_THREAD != 0);
    
    if is_thread {
        // Existing thread implementation
        return sys_clone_thread(flags, stack, parent_tid, tls, child_tid, tf);
    }
    
    // TEAM_312: Fork case - create new process with copied address space
    sys_clone_fork(flags, tf)
}

fn sys_clone_fork(flags: u64, tf: &crate::arch::SyscallFrame) -> i64 {
    let parent = crate::task::current_task();
    
    // 1. Copy address space
    let child_ttbr0 = match mm_user::copy_user_address_space(parent.ttbr0) {
        Some(t) => t,
        None => return errno::ENOMEM,
    };
    
    // 2. Allocate kernel stack for child
    let kernel_stack = alloc_kernel_stack()?;
    
    // 3. Clone parent's syscall frame
    let mut child_frame = *tf;
    child_frame.set_return(0); // Child returns 0
    
    // 4. Create TCB
    let child_pid = Pid::next();
    let child_tcb = create_forked_tcb(parent, child_pid, child_ttbr0, kernel_stack, child_frame);
    
    // 5. Clone FD table
    let child_fds = parent.fd_table.lock().clone();
    child_tcb.fd_table = IrqSafeLock::new(child_fds);
    
    // 6. Register in process table
    process_table::register_process(child_pid.0, parent.id.0, child_tcb.clone());
    
    // 7. Add to scheduler
    scheduler::SCHEDULER.add_task(child_tcb);
    
    // 8. Return child PID to parent
    child_pid.0 as i64
}
```

### 2.2 Exit Criteria
- [ ] Fork creates child process
- [ ] Parent gets child PID
- [ ] Child gets 0
- [ ] Child has independent address space

---

## Step 3 — Exec Implementation

**File**: `kernel/src/syscall/process.rs`

### 3.1 Implement full `sys_exec()`

```rust
/// TEAM_312: sys_exec - Replace current process with new executable.
pub fn sys_exec(path_ptr: usize, path_len: usize) -> i64 {
    let task = crate::task::current_task();
    
    // 1. Validate and copy path
    let mut path_buf = [0u8; 256];
    let path = match crate::syscall::copy_user_string(task.ttbr0, path_ptr, path_len, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    log::trace!("[SYSCALL] exec('{}')", path);
    
    // 2. Find ELF in initramfs
    let elf_data = {
        let archive_lock = crate::fs::INITRAMFS.lock();
        let archive = match archive_lock.as_ref() {
            Some(a) => a,
            None => return errno::ENOSYS,
        };
        
        let lookup_name = path.strip_prefix('/').unwrap_or(path);
        let mut found = None;
        for entry in archive.archive.iter() {
            if entry.name == lookup_name {
                found = Some(alloc::vec::Vec::from(entry.data));
                break;
            }
        }
        
        match found {
            Some(d) => d,
            None => return errno::ENOENT,
        }
    };
    
    // 3. Clear current user address space
    mm_user::clear_user_address_space(task.ttbr0);
    
    // 4. Parse and load new ELF
    let elf = match Elf::parse(&elf_data) {
        Ok(e) => e,
        Err(_) => return errno::ENOEXEC,
    };
    
    let (entry_point, brk) = match elf.load(task.ttbr0) {
        Ok((e, b)) => (e, b),
        Err(_) => return errno::ENOMEM,
    };
    
    // 5. Set up new stack
    let stack_pages = mm_user::layout::STACK_SIZE / PAGE_SIZE;
    let stack_top = match unsafe { mm_user::setup_user_stack(task.ttbr0, stack_pages) } {
        Ok(s) => s,
        Err(_) => return errno::ENOMEM,
    };
    
    // 6. Close O_CLOEXEC file descriptors
    task.fd_table.lock().close_cloexec();
    
    // 7. Update task's brk
    task.heap.lock().set_brk(brk);
    
    // 8. Modify current syscall frame to return to new entry point
    // TEAM_312: Instead of calling a separate function, we modify the trap frame
    // on the kernel stack. When sys_exec returns, the exception return will
    // jump to the new entry point with the new stack.
    unsafe {
        let frame = crate::arch::current_syscall_frame();
        (*frame).set_pc(entry_point as u64);
        (*frame).set_sp(stack_top as u64);
        (*frame).set_return(0); // execve returns 0 to new process
    }
    
    // Return 0 - syscall machinery will restore modified frame
    0
}
```

### 3.2 Exit Criteria
- [ ] Exec loads new ELF
- [ ] Old memory freed
- [ ] New entry point executed
- [ ] Only returns on error

---

## Step 4 — Userspace API

**File**: `userspace/libsyscall/src/process.rs`

### 4.1 Add `fork()` function

```rust
/// TEAM_312: Fork the current process.
///
/// Creates a child process that is a copy of the parent.
///
/// # Returns
/// - `0` to the child process
/// - Child's PID to the parent process  
/// - Negative errno on error
#[inline]
pub fn fork() -> isize {
    // Clone with SIGCHLD only (no CLONE_VM, no CLONE_THREAD)
    const SIGCHLD: u64 = 17;
    clone(SIGCHLD, 0, core::ptr::null_mut(), 0, core::ptr::null_mut())
}
```

### 4.2 Add `exec_args()` function

```rust
/// TEAM_312: Execute with arguments.
///
/// Replaces current process with new executable, passing arguments.
///
/// # Arguments
/// * `path` - Path to executable
/// * `argv` - Command line arguments
///
/// # Returns
/// Only returns on error (negative errno).
#[inline]
pub fn exec_args(path: &str, argv: &[&str]) -> isize {
    // Build ArgvEntry array on stack
    let mut entries = [ArgvEntry { ptr: core::ptr::null(), len: 0 }; 16];
    let argc = argv.len().min(16);
    for (i, arg) in argv.iter().take(argc).enumerate() {
        entries[i] = ArgvEntry { ptr: arg.as_ptr(), len: arg.len() };
    }
    arch::syscall4(
        __NR_execve as u64,
        path.as_ptr() as u64,
        path.len() as u64,
        entries.as_ptr() as u64,
        argc as u64,
    ) as isize
}
```

### 4.3 Update imports in `sysno.rs` (if needed)

Ensure `__NR_clone` and `__NR_execve` are available.

### 4.3 Exit Criteria
- [ ] `fork()` available in libsyscall
- [ ] Userspace can call fork+exec

---

## Step 5 — Integration

### 5.1 Migrate spawn callsites

**Before:**
```rust
let pid = spawn("/bin/shell");
```

**After:**
```rust
let pid = fork();
if pid == 0 {
    // Child
    exec("/bin/shell");
    exit(1); // exec failed
}
// Parent continues with child PID
```

### 5.2 Files to migrate

| File | Change |
|------|--------|
| `userspace/init/src/main.rs` | `spawn()` → `fork()+exec()` |
| `userspace/levbox/test_runner.rs` | `spawn()` → `fork()+exec()` |
| `userspace/levbox/suite_test_core.rs` | `spawn_args()` → `fork()+exec_args()` |
| `userspace/shell/src/main.rs` | `spawn_args()` → `fork()+exec_args()` |

### 5.3 Remove deprecated syscalls

After all callsites migrated:

1. Remove `Spawn` and `SpawnArgs` from `los_abi::SyscallNumber`
2. Remove `sys_spawn()` and `sys_spawn_args()` from kernel
3. Remove from syscall dispatch table

### 5.4 Exit Criteria
- [ ] All spawn callsites migrated
- [ ] Spawn/SpawnArgs removed
- [ ] No deprecation warnings
- [ ] All tests pass

---

## Testing Strategy

### Unit Tests
- `copy_user_address_space()` copies pages correctly
- `clear_user_address_space()` frees all pages
- Fork returns correct values to parent/child

### Integration Tests  
- Fork+exec pattern works
- Shell can spawn commands
- Test runner executes tests

### Golden Tests
- Boot sequence unchanged
- Shell behavior unchanged

---

---

## Handoff Checklist

Before marking implementation complete:

- [ ] Project builds cleanly (`cargo build -p levitate-kernel`)
- [ ] All tests pass (`cargo xtask test --arch x86_64`)
- [ ] Golden boot tests pass
- [ ] All spawn callsites migrated to fork+exec
- [ ] Spawn/SpawnArgs removed from kernel and los_abi
- [ ] No deprecation warnings remain

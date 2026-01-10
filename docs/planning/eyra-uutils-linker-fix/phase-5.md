# Phase 5: Kernel Testing & Validation

**TEAM_368** | Eyra/uutils Integration  
**Created:** 2026-01-10

---

## 1. Phase Summary

**Goal:** Verify Eyra binaries work correctly when run on the LevitateOS kernel.

**Prerequisite:** Phase 4 complete (Eyra binaries in initramfs)

---

## 2. Technical Context

### What Eyra Binaries Are
- **Static PIE Linux binaries** — self-contained, position-independent
- **Pure Rust libc** — no glibc dependency
- **Direct syscalls** — uses Linux syscall interface via `rustix`

### What This Means for LevitateOS
- Binaries expect Linux-compatible syscalls
- The kernel's syscall layer must handle these correctly
- Some syscalls may not be implemented yet

---

## 3. Syscall Requirements

### Critical Syscalls (must work)
| Syscall | Used By | Purpose |
|---------|---------|---------|
| read | all | stdin, file reading |
| write | all | stdout, stderr, file writing |
| open/openat | cat, ls, cp, mv, rm, touch | file operations |
| close | all | resource cleanup |
| exit | all | process termination |
| stat/fstat/lstat | ls, cp, mv, rm | file metadata |
| mmap | all | memory allocation (Eyra's allocator) |
| brk | (maybe) | heap growth |
| getcwd | pwd | current directory |
| chdir | (internal) | directory navigation |
| mkdir | mkdir | create directories |
| rmdir | rmdir | remove directories |
| unlink | rm | remove files |
| rename | mv | rename/move files |
| link/symlink | ln | create links |
| getdents64 | ls | directory listing |

### Eyra-Specific Initialization
Eyra's `origin` crate performs initialization before main():
1. Stack setup
2. Relocations (for PIE)
3. TLS (Thread-Local Storage) setup
4. Allocator initialization

---

## 4. Test Strategy

### 4.1 Basic Execution Test
Verify binaries can start and exit cleanly.

```
# In LevitateOS shell (Eyra binaries are in root, no prefix)
true
echo $?   # Should be 0

false
echo $?   # Should be 1
```

### 4.2 Output Test
Verify stdout works.

```
echo hello world
# Should print: hello world

pwd
# Should print current directory
```

### 4.3 File Operations Test
Verify file syscalls work.

```
touch /tmp/testfile
ls /tmp
cat /tmp/testfile
rm /tmp/testfile
```

### 4.4 Directory Operations Test
```
mkdir /tmp/testdir
ls /tmp
rmdir /tmp/testdir
```

### 4.5 Complex Operations Test
```
cp /etc/passwd /tmp/passwd_copy
mv /tmp/passwd_copy /tmp/passwd_moved
ln /tmp/passwd_moved /tmp/passwd_link
```

---

## 5. Expected Failure Modes

### 5.1 Syscall Not Implemented
**Symptom:** Binary crashes or prints "Function not implemented"
**Fix:** Implement missing syscall in kernel

### 5.2 Syscall Incorrect Behavior
**Symptom:** Binary works but produces wrong results
**Fix:** Debug and fix kernel syscall implementation

### 5.3 Memory Allocation Failure
**Symptom:** Crash during startup (before main)
**Fix:** Ensure mmap works correctly for anonymous mappings

### 5.4 PIE Relocation Failure
**Symptom:** Immediate crash or segfault
**Fix:** Verify kernel correctly loads PIE binaries

---

## 6. Steps

### Step 1: Boot LevitateOS with Eyra initramfs
```bash
cargo xtask run --arch x86_64
```

### Step 2: Run true/false tests
Simplest binaries — verify basic execution works.

### Step 3: Run echo/pwd tests
Verify stdout and basic syscalls.

### Step 4: Run file operation tests
Verify open/read/write/close work.

### Step 5: Document failures
For each failing utility:
- Identify which syscall fails
- Create issue/question for kernel team
- Note workaround if available

### Step 6: Create regression test
Add automated test that runs Eyra binaries.

---

## 7. Known Limitations

### Current Kernel State
The LevitateOS kernel may not implement all Linux syscalls. Expected gaps:
- Advanced file operations (chmod, chown)
- User/group handling
- Some stat fields

### Eyra Binary Expectations
Eyra binaries are designed for Linux. They expect:
- Linux syscall numbers and semantics
- /proc filesystem (for some operations)
- Standard file descriptors (0=stdin, 1=stdout, 2=stderr)

---

## 8. Success Criteria

| Level | Criteria |
|-------|----------|
| **Minimal** | true/false execute and return correct exit codes |
| **Basic** | echo, pwd produce correct output |
| **Functional** | cat, ls, mkdir work for basic cases |
| **Complete** | All 14 utilities work for common use cases |

---

## 9. Open Questions

### Q1: PIE Loading
Does the kernel ELF loader correctly handle static PIE binaries?

### Q2: Syscall Coverage
Which syscalls are already implemented? Need audit of kernel syscall table.

### Q3: Error Reporting
How should the kernel report unimplemented syscalls? (ENOSYS? Panic? Log?)

---

## 10. Dependencies

- **Requires:** Phase 4 complete (binaries in initramfs)
- **Requires:** Kernel boots and can exec binaries
- **May require:** Kernel syscall additions


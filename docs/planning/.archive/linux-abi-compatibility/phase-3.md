# Phase 3: Fix Design and Validation Plan

**TEAM_339** | Linux ABI Compatibility Bugfix

## Root Cause Summary

LevitateOS syscalls use length-counted strings `(ptr, len)` instead of Linux's null-terminated strings. This was a deliberate safety design but breaks Linux ABI.

**Location:** All path-handling syscalls in `crates/kernel/src/syscall/fs/*.rs`

## Fix Strategy

### Approach: Full Linux ABI Compliance

Convert all syscalls to match Linux signatures exactly:

1. **Path arguments:** Accept null-terminated `const char *pathname`
2. **Add missing arguments:** `dirfd` for `*at()` syscalls, `mode` for file creation
3. **Struct layouts:** Use `linux_raw_sys` types or verify exact match
4. **Return values:** Match Linux semantics exactly

### Implementation Pattern

For each path syscall:

```rust
// BEFORE (LevitateOS)
pub fn sys_openat(path: usize, path_len: usize, flags: u32) -> i64

// AFTER (Linux-compatible)
pub fn sys_openat(dirfd: i32, pathname: usize, flags: u32, mode: u32) -> i64 {
    // 1. Handle AT_FDCWD for dirfd
    // 2. Read null-terminated string from pathname
    // 3. Use mode for file creation
    // ...
}
```

### Safe Null-Termination Helper

Create a helper to safely read null-terminated strings:

```rust
/// Read null-terminated string from user memory (max 4096 bytes)
fn read_user_cstring(ttbr0: usize, user_ptr: usize, buf: &mut [u8]) -> Result<&str, i64> {
    for i in 0..buf.len() {
        match user_va_to_kernel_ptr(ttbr0, user_ptr + i) {
            Some(ptr) => {
                let byte = unsafe { *ptr };
                if byte == 0 {
                    return core::str::from_utf8(&buf[..i])
                        .map_err(|_| errno::EINVAL);
                }
                buf[i] = byte;
            }
            None => return Err(errno::EFAULT),
        }
    }
    Err(errno::ENAMETOOLONG)
}
```

## Reversal Strategy

### Rollback Signals
- Tests fail after changes
- Userspace crashes
- Performance degradation

### Rollback Steps
1. Git revert the change batch
2. Rebuild kernel and userspace
3. Verify tests pass

### Checkpoint Tests
After each batch, verify:
```bash
cargo xtask test behavior
cargo test -p libsyscall
```

## Test Strategy

### New Tests Required

| Test | Purpose |
|------|---------|
| `test_openat_linux_abi` | Verify Linux-compatible openat signature |
| `test_null_terminated_paths` | Verify null-termination handling |
| `test_at_fdcwd` | Verify AT_FDCWD support |
| `test_dirfd_resolution` | Verify relative path with dirfd |

### Existing Tests to Update

All tests in `libsyscall` that call path syscalls need updating to pass null-terminated strings.

### Edge Cases

1. Path exactly at max length (4095 bytes)
2. Path with embedded null (should fail)
3. Invalid dirfd values
4. AT_FDCWD constant usage

## Impact Analysis

### API Changes
- All path syscalls get new signatures
- Userspace must update all wrappers
- Any external code using old ABI will break

### Behavior Changes
- Can now use dirfd for relative paths
- Null-terminated strings only (no explicit length)

### Performance
- Slight overhead for null scanning (~negligible for typical paths)
- Removed length validation overhead

---

## Steps

### Step 1: Design Safe String Helper

**Tasks:**
1. Design `read_user_cstring()` function
2. Add `ENAMETOOLONG` error code
3. Add `AT_FDCWD` constant

**Output:** Helper function specification

### Step 2: Design Syscall Signature Changes

**Tasks:**
1. For each affected syscall, document new signature
2. Match to Linux man page exactly
3. Document argument mapping

**Output:** Syscall signature table

### Step 3: Design Test Updates

**Tasks:**
1. List all tests needing updates
2. Design new ABI compliance tests
3. Define test order (run after each batch)

**Output:** Test plan

### Step 4: Define Implementation Batches

**Tasks:**
1. Group syscalls by risk/complexity
2. Define batch order
3. Set checkpoint criteria

**Output:** Batched implementation plan for Phase 4

---

## Exit Criteria

- [ ] Safe string helper designed
- [ ] All syscall signatures documented
- [ ] Test plan complete
- [ ] Implementation batches defined
- [ ] Ready for Phase 4 implementation

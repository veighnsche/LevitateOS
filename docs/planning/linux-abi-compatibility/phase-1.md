# Phase 1: Understanding and Scoping

**TEAM_339** | Linux ABI Compatibility Bugfix

## Bug Summary

LevitateOS syscall interface uses Linux syscall numbers but with custom signatures that differ from Linux ABI. Path-handling syscalls accept `(ptr, len)` instead of null-terminated strings.

**Severity:** High  
**Impact:** Cannot run standard Linux binaries, incompatible with glibc/musl

## Reproduction Status

**Reproducible:** Yes  
**Steps:**
1. Compile a standard Linux binary using glibc
2. Try to run it on LevitateOS
3. Observe syscall failures due to argument mismatch

**Expected:** Syscalls work with Linux-standard arguments  
**Actual:** Syscalls expect LevitateOS-specific argument layout

## Context

### Code Areas Affected

| Area | Files | Issue |
|------|-------|-------|
| Path syscalls | `crates/kernel/src/syscall/fs/*.rs` | Non-Linux signatures |
| Syscall numbers | `crates/userspace/libsyscall/src/sysno.rs` | `__NR_pause` hardcoded |
| Stat struct | `crates/kernel/src/arch/*/mod.rs` | Custom vs linux_raw_sys |
| Error codes | `crates/kernel/src/syscall/mod.rs` | Duplicate definitions |

### Recent Changes
None specific - this is a foundational design issue.

## Constraints

- **Backwards Compatibility:** Must update userspace libsyscall to match
- **Platforms:** Both x86_64 and aarch64 affected
- **Time Sensitivity:** Not urgent, but blocks Linux binary compatibility goal

## Open Questions

1. ~~Is Linux binary compatibility a goal?~~ **Assumed YES for this plan**
2. Should null-terminated string handling use safe wrappers?
3. How to handle the aarch64 `pause` syscall (doesn't exist in Linux aarch64)?

---

## Steps

### Step 1: Complete Discrepancy Inventory

**File:** `phase-1-step-1.md`

Systematically catalog every syscall signature difference.

**Tasks:**
1. List all syscalls in kernel dispatcher
2. Compare each signature to Linux man pages
3. Document exact argument mismatches

**Output:** Complete table of discrepancies in `discrepancies.md`

### Step 2: Struct Layout Verification

**File:** `phase-1-step-2.md`

Verify struct sizes match between kernel and Linux.

**Tasks:**
1. Check `Stat` struct size (kernel vs linux_raw_sys)
2. Check `Termios` struct size
3. Check `Timespec` struct size
4. Document any mismatches

**Output:** Struct comparison table

### Step 3: Identify Userspace Impact

**File:** `phase-1-step-3.md`

Map all userspace code that needs updating.

**Tasks:**
1. List all libsyscall wrapper functions
2. Note which need signature changes
3. List all levbox/init code calling syscalls

**Output:** Userspace change list

---

## Exit Criteria

- [ ] Complete inventory of all discrepancies
- [ ] Struct layouts verified
- [ ] Userspace impact documented
- [ ] Ready for Phase 2 classification

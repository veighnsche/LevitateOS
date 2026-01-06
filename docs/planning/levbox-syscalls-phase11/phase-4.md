# Phase 4: Integration — Writable Filesystem for Levbox

**Phase:** Integration  
**Status:** ✅ COMPLETE  
**Team:** TEAM_194, TEAM_195

---

## Prerequisites

- [x] Phase 3 implementation complete ✅
- [x] All syscalls working individually ✅

---

## Integration Steps

### Step 1: Initialize Tmpfs at Boot

**File:** `kernel/src/init.rs` or `kernel/src/fs/mod.rs`

**Tasks:**
1. Call `Tmpfs::new()` during kernel init
2. Create `/tmp` directory automatically
3. Log tmpfs initialization

**Exit Criteria:**
- Tmpfs ready before first userspace process

---

### Step 2: Update Shell for /tmp

**File:** `userspace/shell/src/main.rs`

**Tasks:**
1. Verify `cd /tmp` works (if chdir implemented)
2. Test built-in commands with /tmp paths
3. Test command execution from /tmp (if applicable)

**Exit Criteria:**
- Shell can interact with /tmp

---

### Step 3: Verify Levbox Utilities

**Test each utility:**

| Utility | Test Command | Expected Result |
|---------|--------------|-----------------|
| mkdir | `mkdir /tmp/test` | Directory created |
| rmdir | `rmdir /tmp/test` | Directory removed |
| touch | `touch /tmp/file` | File created |
| rm | `rm /tmp/file` | File removed |
| cp | `cp /init /tmp/init` | File copied |
| mv | `mv /tmp/a /tmp/b` | File renamed |
| ls | `ls /tmp` | Shows contents |
| cat | `cat /tmp/file` | Shows content |

**Exit Criteria:**
- All levbox utilities work with /tmp

---

### Step 4: Cross-Filesystem Operations

**Test:**
1. `cp /init /tmp/init` — copy from initramfs to tmpfs
2. `cat /tmp/init` — verify content matches
3. `mkdir /etc/foo` — should return EROFS

**Exit Criteria:**
- Cross-fs copy works
- Initramfs remains read-only

---

### Step 5: Error Handling

**Test edge cases:**

| Test | Expected |
|------|----------|
| `rm /tmp/nonexistent` | ENOENT |
| `rmdir /tmp/nonempty` | ENOTEMPTY |
| `mkdir /tmp/existing` | EEXIST |
| `mv /init /tmp/init` | EXDEV (cross-fs) |
| Create file > max size | ENOSPC or EFBIG |

**Exit Criteria:**
- All error cases handled correctly

---

## Verification

Run full test suite after integration:
```bash
cargo xtask test
```

All tests must pass.

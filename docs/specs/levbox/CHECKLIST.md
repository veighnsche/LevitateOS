# Levbox Implementation Checklist

Status of the "Busybox" core utilities for LevitateOS (Phase 11).

## 🛠️ Syscall Readiness

These syscalls are required by the utilities below. Status tracks kernel support + `libsyscall` wrapper.

| Syscall | Required By | Wrapper | Kernel | Notes |
|---------|-------------|---------|--------|-------|
| `openat` | cat, ls, cp, touch | 🟢 | 🟢 | Read-only + tmpfs write via O_CREAT |
| `read` | cat, cp | 🟢 | 🟢 | Supports tmpfs files |
| `write` | cat, cp | 🟢 | 🟢 | stdout/stderr + tmpfs files |
| `close` | All | 🟢 | 🟢 | |
| `fstat` | ls, cp, rm, mv | 🟢 | 🟢 | Supports tmpfs files |
| `getdents64` | ls, cp, rm | 🟢 | 🟢 | |
| `getcwd` | pwd | 🟢 | 🟢 | |
| `mkdirat` | mkdir, cp | 🟢 | 🟢 | Works for `/tmp/*` paths (tmpfs) |
| `unlinkat` | rmdir, rm | 🟢 | 🟢 | Works for `/tmp/*` paths (tmpfs) |
| `renameat` | mv | 🟢 | 🟢 | Works for `/tmp/*` paths (tmpfs) |
| `linkat` | ln | 🔴 | 🔴 | **BLOCKER**: Full implementation needed |
| `symlinkat` | ln | 🔴 | 🔴 | **BLOCKER**: Full implementation needed |
| `utimensat` | touch | 🔴 | 🔴 | **BLOCKER**: Full implementation needed |

---

## 📦 Utility Feature Checklist

### [cat](cat.md) (🟢 Complete)
- [x] Basic concatenation (`cat file`)
- [x] Multiple files (`cat f1 f2`)
- [x] Stdin support (`cat`, `cat -`)
- [x] Unbuffered mode (`-u`)
- [x] Help output (`--help`)
- [x] Version output (`--version`)
- [x] Exit codes (0 on success, 1 on error)

### [ls](ls.md) (� Complete)
- [x] Basic listing
- [x] Hidden files (`-a`, `-A`)
- [x] Long listing (`-l`)
- [x] Human readable sizes (`-h`)
- [x] Type indicators (`-F`)
- [x] Recursive listing (`-R`)
- [x] Single column output (`-1`)
- [x] Help and Version
- [x] Sorting (Alphabetical)
- [x] Color output (`--color`)

### [mkdir](mkdir.md) (� In Progress)
- [x] Create directory
- [ ] Parent directories (`-p`) - option parsed, not implemented
- [ ] Mode setting (`-m`) - option parsed, not implemented
- [x] Verbose output (`-v`)
- [x] Help and Version

### [rmdir](rmdir.md) (� In Progress)
- [x] Remove empty directory
- [ ] Parent directories (`-p`) - option parsed, not implemented
- [x] Verbose output (`-v`)
- [x] Help and Version

### [rm](rm.md) (� In Progress)
- [x] Remove files
- [x] Force remove (`-f`)
- [ ] Interactive remove (`-i`) - not implemented
- [ ] Recursive remove (`-r`, `-R`) - option parsed, not implemented
- [x] Remove empty directories (`-d`)
- [x] Help and Version

### [pwd](pwd.md) (� Complete)
- [x] Print working directory
- [x] Physical path (`-P`)
- [x] Logical path (`-L`)
- [x] Help and Version

### [touch](touch.md) (🔴 Planned)
- [ ] Create empty file
- [ ] Update access time (`-a`)
- [ ] Do not create file (`-c`)
- [ ] Update modification time (`-m`)
- [ ] Reference file timestamps (`-r`)
- [ ] Specific timestamp (`-t`)
- [ ] Help and Version

### [cp](cp.md) (� In Progress)
- [ ] Copy files
- [x] Force copy (`-f`) - option parsed
- [x] Interactive copy (`-i`) - option parsed
- [x] Preserve attributes (`-p`) - option parsed
- [x] Recursive copy (`-R`) - option parsed
- [x] Help and Version

### [mv](mv.md) (� In Progress)
- [x] Rename/Move files
- [x] Force move (`-f`)
- [ ] Interactive move (`-i`) - not implemented
- [x] Help and Version

### [ln](ln.md) (🔴 Planned)
- [ ] Create hard links
- [ ] Create symbolic links (`-s`)
- [ ] Force link (`-f`)
- [ ] Help and Version

---

## ✅ Resolved Blockers

**Updated:** 2026-01-06 (TEAM_194, TEAM_195)

### Tmpfs Implementation Complete

TEAM_194 implemented tmpfs (in-memory writable filesystem) at `/tmp`:

| Feature | Status | Notes |
|---------|--------|-------|
| `mkdirat` for `/tmp/*` | 🟢 Complete | `mkdir /tmp/foo` works |
| `unlinkat` for `/tmp/*` | 🟢 Complete | `rm /tmp/file`, `rmdir /tmp/dir` work |
| `renameat` for `/tmp/*` | 🟢 Complete | `mv /tmp/a /tmp/b` works |
| `openat` with O_CREAT | 🟢 Complete | Creates files in `/tmp` |
| `openat` with O_TRUNC | 🟢 Complete | Truncates files in `/tmp` |
| `read`/`write` for tmpfs | 🟢 Complete | Full read/write support |
| `fstat` for tmpfs | 🟢 Complete | Returns file size and mode |

**Tmpfs Limits:**
- Max file size: 16MB
- Max total size: 64MB

---

## ⚠️ Remaining Blockers

### Priority 1: New Syscalls Needed

These syscalls have **no wrapper and no kernel support**:

| Syscall | Number | Unblocks | Implementation Notes |
|---------|--------|----------|----------------------|
| `linkat` | 37 | ln | Create hard links |
| `symlinkat` | 36 | ln -s | Create symbolic links |
| `utimensat` | 88 | touch | Set file access/modification times |

### ✅ Tmpfs Initialization (Complete)

| Task | Status | Notes |
|------|--------|-------|
| Call `tmpfs::init()` at boot | � Complete | Added by TEAM_195 in `init_filesystem()` |

### Next Steps

1. ~~**Add `tmpfs::init()` call** to kernel initialization~~ ✅ Done
2. **Test levbox utilities** with `/tmp` paths (run QEMU)
3. **Add linkat, symlinkat, utimensat** syscalls for `ln` and `touch`

See also: `docs/ROADMAP.md` Phase 11 section.

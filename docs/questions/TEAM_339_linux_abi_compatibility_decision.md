# Question: Linux ABI Compatibility

**TEAM_339** | 2026-01-09

## Context

Investigation revealed that LevitateOS uses Linux syscall **numbers** but has **custom syscall signatures** that differ from Linux ABI.

Key differences:
- Path arguments pass `(ptr, len)` instead of null-terminated strings
- `openat` missing `dirfd` parameter
- `__NR_pause` hardcoded for x86_64 only
- Kernel `Stat` struct may differ from `linux_raw_sys::general::stat`

## Decision Required

**Is LevitateOS intended to be Linux binary-compatible?**

### Option A: Full Linux Compatibility
- Refactor all syscalls to match Linux signatures exactly
- Accept null-terminated strings
- Match struct layouts to Linux exactly
- **Effort:** ~20-30 UoW
- **Benefit:** Can run unmodified Linux ELF binaries

### Option B: LevitateOS-Specific ABI (Current State)
- Document that LevitateOS has its own ABI
- Continue with length-counted strings
- Custom userspace only
- **Effort:** 1-2 UoW (documentation)
- **Benefit:** No code changes, safer string handling

### Option C: Hybrid Compatibility Layer
- Keep internal LevitateOS syscalls
- Add translation layer for Linux-compatible syscalls
- **Effort:** ~10-15 UoW
- **Benefit:** Gradual migration path

## Implications

| Aspect | Option A | Option B | Option C |
|--------|----------|----------|----------|
| Run Linux binaries | ✅ Yes | ❌ No | ⚠️ Partial |
| Code changes | Major | None | Medium |
| Safety (buffer handling) | Lower | Higher | Medium |
| Complexity | High | Low | Medium |

## User Input Needed

Please choose A, B, or C, or provide alternative direction.

---

## ✅ ANSWERED: 2026-01-09

**User chose: Option A - Full Linux ABI Compatibility**

Implementation will proceed with the full Linux ABI compatibility plan.

---

**See:** `.teams/TEAM_339_investigate_linux_abi_compatibility.md` for full investigation details.

# Questions: brush Shell Migration

**TEAM_391** | Created: 2026-01-10

## Q1: Tokio/Epoll Resolution Strategy

**Status**: ✅ ANSWERED

brush shell requires tokio async runtime, which requires epoll syscalls that are **NOT implemented** in LevitateOS kernel.

### Options

| Option | Effort | Benefit | Risk |
|--------|--------|---------|------|
| **A: Implement epoll** | 2-4 days | Full tokio support, benefits other apps | Medium complexity |
| **B: Use poll() backend** | 1-2 days | Simpler | May not work, untested |
| **C: Fork brush, remove tokio** | 1-2 weeks | No kernel changes | High maintenance burden |
| **D: Use simpler shell** | 1-2 days | Quick solution | Loses bash compatibility |

**Recommendation**: Option A (implement epoll) - pays dividends for future async apps.

**User decision**: ✅ **Option A: Implement epoll in kernel**

---

## Q2: Shell Binary Name

**Status**: ✅ ANSWERED

What should the binary be named?
- `brush` - Use upstream name ✅
- `sh` - POSIX convention
- `bash` - For script compatibility
- `shell` - Match current LevitateOS convention

**User decision**: ✅ **`brush`** (use upstream name)

# TEAM_469: Procfs Design Questions

All questions resolved - proceeding with implementation.

## Resolved Questions

### Q9: How to get process executable path?

**Context**: `/proc/[pid]/exe` should be a symlink to the executable.

**Decision**: Option 2 - Return `[unknown]` symlink target for now.

Future enhancement: Add `exe_path` field to TCB.

---

### Q10: How to get process command line?

**Context**: `/proc/[pid]/cmdline` should contain null-separated argv.

**Decision**: Option 2 - Return empty string for now.

Future enhancement: Add `cmdline` field to TCB.

---

### Q11: Should /proc/[pid]/fd/[n] show actual file paths?

**Decision**: Option 3 - Return generic names like `[unknown]`, `pipe:[ino]`, `socket:[ino]`.

---

### Q12: Thread visibility in /proc?

**Decision**: Option 1 - Show only main process PIDs.

---

### Q13: What memory stats for /proc/meminfo?

**Decision**: Minimum set - `MemTotal` and `MemFree` only.

---

### Q14: What for /proc/uptime?

**Decision**: Option 1 - Return `uptime 0.0` (no idle tracking).

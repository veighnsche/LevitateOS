# Phase 2 — Design

## Command Design

### 1. `cargo xtask shell start [--arch ARCH]`

**Purpose:** Start VM in background and keep it alive.

**Behavior:**
1. Check if session already exists (read `.qemu-session.json`)
2. If exists, error: "Session already running (PID xxx)"
3. Build kernel/userspace if needed
4. Start QEMU with QMP socket enabled
5. Save session state to `.qemu-session.json`:
   ```json
   {"pid": 12345, "qmp_socket": "./qemu-session.sock", "arch": "aarch64"}
   ```
6. Print connection info and exit (QEMU runs in background)

---

### 2. `cargo xtask shell send "<text>"`

**Purpose:** Send text as keystrokes to running VM.

**Behavior:**
1. Load session from `.qemu-session.json`
2. Connect to QMP socket
3. For each character in text:
   - Translate to QEMU qcode
   - Send via QMP `sendkey`
4. After text, send `ret` (Enter key)

**Keycode translation:**
```
a-z  → "a" through "z"
A-Z  → shift+"a" through shift+"z"
0-9  → "0" through "9"
space → "spc"
enter → "ret"
-    → "minus"
.    → "dot"
/    → "slash"
```

---

### 3. `cargo xtask shell screenshot [--output FILE]`

**Purpose:** Take screenshot of running VM.

**Behavior:**
1. Load session from `.qemu-session.json`
2. Connect to QMP socket
3. Execute `screendump` with filename
4. Print confirmation

---

### 4. `cargo xtask shell stop`

**Purpose:** Kill the running VM session.

**Behavior:**
1. Load session from `.qemu-session.json`
2. Kill the PID
3. Remove `.qemu-session.json`
4. Remove QMP socket file

---

## Session State File

**Location:** `.qemu-session.json` (project root)

**Schema:**
```json
{
  "pid": 12345,
  "qmp_socket": "./qemu-session.sock",
  "arch": "aarch64",
  "started_at": "2024-01-09T09:00:00Z"
}
```

---

## Keycode Translation Table

| Char | Qcode | Notes |
|------|-------|-------|
| a-z | `a` - `z` | Direct |
| A-Z | `shift+a` - `shift+z` | With shift |
| 0-9 | `0` - `9` | Direct |
| ` ` | `spc` | Space |
| `\n` | `ret` | Enter |
| `-` | `minus` | |
| `=` | `equal` | |
| `.` | `dot` | |
| `/` | `slash` | |
| `_` | `shift+minus` | |
| `"` | `shift+apostrophe` | |

---

## Design Decisions

### Q1: Should `shell send` auto-add Enter key?

**Decision:** Yes, automatically append `ret` after text.  
**Rationale:** Most commands need Enter to execute.  
**Override:** Add `--no-enter` flag if needed later.

### Q2: How to handle QEMU crash/exit?

**Decision:** `shell send` and `screenshot` will detect dead process and error cleanly.  
**Rationale:** User can run `shell stop` to clean up state, then `shell start` again.

### Q3: Should we show QEMU output in terminal?

**Decision:** No — QEMU runs headless. Use VNC or screenshot for visual.  
**Rationale:** Keeps the commands simple and scriptable.

---

## Architecture

```
┌─────────────────┐
│   shell start   │──▶ Spawns QEMU (background)
└─────────────────┘         │
                            ▼
                    ┌───────────────┐
                    │ .qemu-session │ (JSON state file)
                    │    .json      │
                    └───────────────┘
                            ▲
┌─────────────────┐         │
│   shell send    │──▶ Reads state, connects QMP, sends keys
└─────────────────┘
                            
┌─────────────────┐         
│ shell screenshot│──▶ Reads state, connects QMP, screendump
└─────────────────┘

┌─────────────────┐
│   shell stop    │──▶ Reads state, kills PID, removes files
└─────────────────┘
```

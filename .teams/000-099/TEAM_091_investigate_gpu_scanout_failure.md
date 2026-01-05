# TEAM_091: Investigate GPU Scanout Failure

## 1. Pre-Investigation Checklist

### 1.1 Registered Team
- **Team ID:** TEAM_091
- **Focus:** GPU Display Scanout Bug

### 1.2 Gather the Bug Report
The user is reporting a bug related to GPU display scanout. Previous teams (TEAM_088, TEAM_089) have been investigating this. The symptom seems to be related to failures during scanout or reset operations in the GPU driver (likely `console_gpu.rs`).

### 1.3 Check Existing Context
- `docs/planning/gpu-display-bugfix`: Planning dir.
- `.teams/TEAM_088_investigate_gpu_display_scanout.md`: Previous investigation effort.
- `.teams/TEAM_089_investigate_gpu_scanout_reset.md`: Another investigation effort.
- `kernel/src/console_gpu.rs`: The likely location of the bug.

---

## 2. Phase 1 — Understand the Symptom

### Symptom Description
The kernel hangs during Stage 4 boot. Serial output stops after `BOOT_REGS`. This was traced to a deadlock/flood in the GPU driver.

---

## 3. Phase 2 — Form Hypotheses
1. **Flush Flood:** Too many `gpu.flush()` calls during boot flood the VirtIO queue.
2. **Deadlock:** Holding `IrqSafeLock` during blocking `flush()` hangs the single-core system.
3. **Byte Order:** `scroll_up` used incorrect color byte order (RGBA vs BGRA).

---

## 4. Phase 3 — Test Hypotheses with Evidence
1. **Evidence:** Removing `flush()` from `write_str` fixed the hang and allowed the system to boot to shell.

---

## 5. Phase 4 — Narrow Down to Root Cause
Flooding the GPU with flush commands while holding a global lock caused the system to hang.

---

## 6. Phase 5 — Decision: Fix or Plan
**Decision: IMPLEMENTED FIX**
- Removed redundant flush in `console_gpu.rs`.
- Fixed byte order in `terminal.rs`.
- Reliability on 10Hz timer flush confirmed.

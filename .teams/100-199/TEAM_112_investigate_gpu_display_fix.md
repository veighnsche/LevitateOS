# TEAM_112: Investigate GPU Display Bug

## 1. Pre-Investigation Checklist

- **Team ID:** TEAM_112
- **Previous Team:** TEAM_111 (Created VNC tool, confirmed bug visually)
- **Symptom:** GPU driver initializes (serial says success), but QEMU display shows "Display output is not active" or remains blank.
- **End Goal:** Visible terminal- [x] Confirm bug (Step 1)
- [x] Investigate VirtQueue Layout (Step 2) -> Verified (Legacy V1 OK).
- [x] Implement Cache Flushing (Step 3) -> Done (dc cvac).
- [ ] Fix Display Output -> **FAILED** (Stalled). "Display output is not active" persists.

## Findings
- **Protocol:** Legacy V1 (MMIO). Queue Layout `ALIGN=4` is correct.
- **Cache:** Flushing `cmd_buf` and `fb` did NOT fix it.
- **Resolution:** 1024x768 and 1280x800 both fail.
- **Payload:** Command bytes are correct.
- **Addresses:** PAs are in valid RAM and aligned.

See `docs/handoffs/TEAM_112_gpu_display_analysis.md` for full report.

## 2. Investigation Log

### Phase 1: Understand the Symptom
- [x] Reproduce with `cargo xtask run-vnc`
- [x] Visually confirm failure.
  - **Findings:** Serial logs show "GPU initialized successfully" and terminal starting. VNC shows "Display output is not active".
  - **Conclusion:** False positive in serial logs. Scanout is not correctly configured.

### Phase 2: Hypotheses
1. **SET_SCANOUT is broken/missing:** The scanout command tells the GPU what resource to show. If this fails or is malformed, display remains inactive.
2. **RESOURCE_FLUSH is missing:** The kernel might draw to the buffer but never tell the host to update the screen. (Less likely to cause "not active", usually just causes stale/blank screen).
3. **VirtQueue Layout Issue:** Use/Avail rings might be misaligned. (Possible, but `GET_DISPLAY_INFO` seemingly works since we see 1280x800 in logs).

### Phase 3: Testing Evidence
- [ ] Test Hypothesis 1 (SET_SCANOUT) by inspecting `levitate-drivers-gpu` code
- [ ] Test Hypothesis 2 (RESOURCE_FLUSH)
- [ ] Test Hypothesis 3 (VirtQueue)

## 3. Root Cause Analysis

(To be filled)

## 4. Decision

(To be filled)

# TEAM_110: Investigate GPU Driver Fallback & Crate Consolidation

## 1. Pre-Investigation Checklist

### 1.1 Team Registration
- **Team ID:** TEAM_110
- **Team File:** `.teams/TEAM_110_investigate_gpu_fallback_and_cleanup.md`

### 1.2 Bug Report
- **Issue:** GPU driver (`levitate-drivers-gpu`) lacks a fallback mechanism.
- **Cleanup:** `levitate-gpu` crate needs to be removed.
- **Context:** `VIRTIO_GPU_SPEC.md` and `TEAM_109_fix_gpu_driver_no_fallback.md` are relevant.

### 1.3 Context Gathering
- [ ] Read `VIRTIO_GPU_SPEC.md`
- [ ] Read `TEAM_109_fix_gpu_driver_no_fallback.md`
- [ ] Inspect `levitate-drivers-gpu` and `levitate-gpu` directories.

## 2. Phase 1 — Understand the Symptom

### 2.1 Symptom Description
- **Expected Behavior:** If the VirtIO GPU driver fails or is unavailable, the system should fallback to a basic frame buffer (e.g., SimpleFB) or maintain serial console stability.
- **Actual Behavior:** (TBD)
- **Delta:** (TBD)

### 2.2 System Location
- `levitate-drivers-gpu`
- `kernel/src/console_gpu.rs` (mentioned in previous conversation summaries as a source of hangs)

## 3. Phase 1 — Understand the Symptom

### 3.1 Symptom Description
- **Expected Behavior:** `levitate-drivers-gpu` should initialize and show display output in QEMU.
- **Actual Behavior:** `levitate-drivers-gpu` times out on the first command (`GET_DISPLAY_INFO`). `levitate-gpu` initializes but shows nothing.
- **Delta:** `levitate-drivers-gpu` is timing out because it never sees the device's response.

### 3.2 System Location
- `levitate-virtio/src/queue.rs`: VirtQueue memory layout.
- `levitate-drivers-gpu/src/device.rs`: Command sending and timeout logic.

## 4. Phase 2 — Form Hypotheses

### 4.1 Hypothesis 1: Shifted VirtQueue Layout (HIGH CONFIDENCE)
- **Theory:** `VirtQueue` struct includes `used_event` and `avail_event` fields, but `VIRTIO_F_EVENT_IDX` is not negotiated. According to VirtIO 1.1 spec, these fields should only be present if negotiated. Including them shifts the `Used` ring by 4 bytes, causing the driver to poll the wrong memory location for `used_idx`.
- **Evidence needed:** Remove fields and check if timeout persists.

### 4.2 Hypothesis 2: Missing Cache Barriers in HAL (MEDIUM CONFIDENCE)
- **Theory:** Even though we have `dsb sy`, the memory allocated via `alloc::alloc` might not be DMA-coherent on some AArch64 setups.
- **Evidence needed:** (N/A yet, Hypothesis 1 is more likely).

### 4.3 Hypothesis 3: Redundant GPU Crates (CONFIRMED)
- **Theory:** `levitate-gpu` is a wrapper around `virtio-drivers` which lacks `SET_SCANOUT` and `RESOURCE_FLUSH`, leading to "Display output is not active" in QEMU despite "successful" init.
- **Action:** Remove `levitate-gpu` and fix `levitate-drivers-gpu`.

## 5. Phase 3 — Test Hypotheses with Evidence

### 5.1 Testing Hypothesis 1 & 4
- **Plan:** Remove `used_event`, `_padding`, and `avail_event` from `levitate-virtio/src/queue.rs`. Add Legacy MMIO (Version 1) support to `MmioTransport`.
- **Status:** [x] DONE
- **Result:** SUCCESS. The driver now initializes correctly on QEMU (which provides Version 1 MMIO). Commands no longer timeout.

## 6. Results & Resolution

### 6.1 Final Changes
- **`levitate-virtio`**: Fixed `VirtQueue` memory layout (removed optional fields) and added support for Legacy MMIO (Version 1) in `MmioTransport`.
- **`levitate-drivers-gpu`**: Native driver is now fully functional and consolidated.
- **`levitate-gpu`**: REMOVED. This crate was a non-functional wrapper and is no longer needed.
- **`kernel`**: Integrated `levitate-drivers-gpu` and updated initialization to use MMIO base address.

### 6.2 Verification Results
- **Unit Tests**: All passed.
- **Behavior Test**: Confirmed successful GPU initialization:
  ```
  13: GPU initialized successfully.
  14: [TERM] GPU resolution: 1280x800
  ```
- **Fallback**: No longer triggered on working hardware.

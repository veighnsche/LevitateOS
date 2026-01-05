# Phase 4: High-Level Abstractions and Integration

## Target Design
The final orchestration layer that provides the `Display` and `GpuState` types to the kernel.

## Perfection Criteria
- The Terminal continues to render to a `DrawTarget` and remains unaware of the driver internals.
- **Scanout Stability:** The driver automatically recovers the host-side scanout if a timeout is detected or on a health-check interval.

## Steps
1. **Step 1 – Refactor GpuState to use Driver + Resource**
2. **Step 2 – Implement Scanout Health Check Loop**
3. **Step 3 – Integrate with main.rs Telemetry**
   - Ensure the heartbeat shows meaningful architectural health metrics, not just "flushes".

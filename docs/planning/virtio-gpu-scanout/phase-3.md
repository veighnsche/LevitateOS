# Phase 3: Driver and State Machine

## Target Design
Implement the core `VirtioGpuDriver` that manages the transport and command queue.

## Perfection Criteria
- **Async-first design** per user requirement (Q4: "DO IT RIGHT FROM THE START!!!").
- Decouple the "how to talk to VirtIO" from the "what to draw".
- Precise error handling for every command-response cycle.
- Documented `unsafe` blocks for MMIO and DMA access.

## Steps
1. **Step 1 – Implement Async Command Queue Manager**
   - Handles the request/response pairing via VirtQueues.
   - Uses `core::task::Waker` for async completion notification.
   - Provides both `async fn send()` and blocking `send_blocking()` wrapper.
2. **Step 2 – Implement Driver Initialization**
   - Port the 2D device initialization logic.
   - State machine: `Uninitialized → ResourceCreated → BackingAttached → ScanoutSet → Ready`.
3. **Step 3 – Add Structured Observability**
   - Provide telemetry hooks for tracking flushes, errors, and command latency.
   - Log every command/response with parsed CtrlType.

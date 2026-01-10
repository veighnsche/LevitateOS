# NVMe Detailed Design

This document outlines the implementation plan for the NVMe storage driver, drawing architectural inspiration from **Theseus** and its high-performance queue-based drivers (`mlx5`).

## Architectural Inspiration (Theseus/Mellanox)

The NVMe driver will follow the "Submission/Completion Queue (SQ/CQ)" pattern found in modern NICs and storage controllers:

1.  **Contiguous DMA Memory**: Queues must be physically contiguous. We will use the HAL's `allocate_contiguous` (or similar) to ensure the driver doesn't cross page boundaries for queue entries.
2.  **Doorbell Registers**: Interaction with the hardware is minimized to "ringing a doorbell" (MMIO write) once a batch of commands is added to the SQ.
3.  **Ownership Model**: The driver "owns" the memory of the queues, while the hardware "borrows" it via DMA. We use memory barriers or atomic writes (Rule 7) to ensure coherency.

## Component: `crates/drivers/nvme`

### 1. Controller Structure
```rust
pub struct NvmeController {
    bar0: DeviceRegion, // Mapped via HAL DeviceMapper
    admin_sq: NvmeQueue<SubmissionEntry>,
    admin_cq: NvmeQueue<CompletionEntry>,
    io_queues: Vec<NvmeIoQueuePair>,
    capabilities: ControllerCaps,
}
```

### 2. Queue Mechanism
We will implement a generic `NvmeQueue<T>` that handles:
- Circular indexing (Head/Tail pointers).
- Phase bits for Completion Queues (to avoid clearing the whole queue every time).
- Virtual-to-Physical translation for the hardware to find the queue.

### 3. Initialization Sequence
Following the standard NVMe handshake (and Theseus style):
1.  **Discovery**: PCI scan finds `01/08/02` (NVMe Controller).
2.  **Mapping**: HAL maps BAR0 (MMIO).
3.  **Reset**: Write `CC.EN = 0`, wait for `CSTS.RDY = 0`.
4.  **Admin Setup**: 
    - Allocate Admin SQ/CQ (Physically Contiguous).
    - Write addresses to `ASQ` and `ACQ` registers.
    - Set queue sizes in `AQA`.
5.  **Enable**: Write `CC.EN = 1`, wait for `CSTS.RDY = 1`.
6.  **I/O Queues**: Use Admin commands to create at least one pair of I/O queues.

## Non-Blocking Strategy (Rule 9)
Initial implementation will use **Polling on Phase Bits**. 
The `read_blocks` future will:
1.  Push command to SQ.
2.  Ring Doorbell.
3.  `yield` or Loop until the CQ entry's "Phase" bit flips.

This keeps the driver simple (Rule 20) but ready for MSI-X interrupts later.

## Verification Plan
- **Mocking**: Create a `MockNvme` struct in `systest` that simulates the phase bit flip.
- **QEMU**: Run with `-device nvme,drive=hd0,serial=foo`.
- **Bare Metal**: Deploy to NUC7 and verify "Controller Ready" serial output.

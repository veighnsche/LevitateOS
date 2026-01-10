# xHCI Detailed Design

This document outlines the implementation plan for the xHCI (USB 3.0) driver, focusing on a minimal stack for HID support on the NUC7i3BNH.

## Architectural Inspiration (crab-usb / rust-osdev)

The xHCI driver will utilize a "Ring and Context" model, which is the standard for USB 3.0 controllers:

1.  **Rings (Circular Queues)**:
    *   **Command Ring**: Used by the driver to send commands to the controller (e.g., Enable Slot, Address Device).
    *   **Event Ring**: Used by the controller to send completion events/interrupts to the driver.
    *   **Transfer Ring**: Each endpoint (EP) of a device has its own transfer ring for data I/O.
2.  **Contexts**:
    *   **Device Context**: Stores the state of a connected USB device (speed, port, etc.).
    *   **Input Context**: Used during "Address Device" or "Configure Endpoint" commands to tell the controller how to set up the device.
3.  **DCBAA (Device Context Base Address Array)**: A shared memory array where the controller finds the pointers to all active device contexts.

## Component: `crates/drivers/xhci`

### 1. Controller Structure
```rust
pub struct XhciController {
    mmio: DeviceRegion, // Mapped via HAL DeviceMapper (Cap/Op/Runtime regs)
    command_ring: XhciRing,
    event_ring: XhciEventRing,
    dcbaa: Box<[u64; 256]>, // Device Context Base Address Array
    slots: Vec<UsbSlot>,
}
```

### 2. Ring Mechanism
We will implement an `XhciRing` that handles:
- **TRB (Transfer Request Block)** allocation.
- **Cycle Bit** management (used by the hardware to distinguish new entries).
- **Enqueue/Dequeue** pointers.
- Physical address translation for the `ERST` (Event Ring Segment Table).

### 3. Minimal HID Stack (Phase 3)
To support a keyboard/mouse on the NUC:
1.  **Discovery**: PCI scan finds `0C/03/30`.
2.  **Reset/Init**:
    *   Reset controller.
    *   Set `MaxDeviceSlots`.
    *   Set `DCBAAP`.
    *   Initialize Command/Event rings.
    *   Enable controller (`USBCMD.RS = 1`).
3.  **Enumeration**:
    *   Wait for "Port Status Change" event.
    *   Issue "Enable Slot" command.
    *   Issue "Address Device" command with Input Context.
    *   Read Device/Config/Interface/Endpoint descriptors.
4.  **HID Polling**:
    *   Identify HID interface and Interrupt IN endpoint.
    *   Push a Transfer TRB to the EP's Transfer Ring.
    *   Ring the Doorbell for that slot/EP.
    *   Wait for "Transfer Event" in the Event Ring.

## Non-Blocking Strategy (Rule 9)
Like NVMe, we will start with **Polling on Event Ring Dequeue Pointer**. 
The `poll` function in `InputDevice` will:
1.  Check the "Cycle Bit" of the current entry in the Event Ring.
2.  If flipped, process the event (Transfer Event, Command Completion, Port Status Change).
3.  Update the `ERDP` (Event Ring Dequeue Pointer) to acknowledge.

## Verification Plan
- **Mocking**: Use `MockXhci` that simulates TRB completion in the Event Ring.
- **QEMU**: Run with `-device qemu-xhci -device usb-kbd`.
- **Bare Metal**: Verify keyboard input on NUC7.

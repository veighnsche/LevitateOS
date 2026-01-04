# Task 6.4: VirtIO Keyboard Support

**Feature**: Phase 6 — VirtIO Ecosystem Expansion
**Status**: Planning
**Priority**: High (Critical for Terminal interaction)

## Objective
Implement a `virtio-keyboard` driver to allow direct user input from the QEMU window (GTK/SDL) to the GPU terminal, removing the dependency on UART echo for terminal interaction.

## Background
Currently, the GPU terminal relies on UART input echo in `kmain`. This is suboptimal as it doesn't provide a true "console" experience and lacks support for standard keyboard events (modifiers, function keys, etc.).

## Requirements
1. **Device Discovery**: Detect `virtio-input-pci` or `virtio-input-device` with keyboard subtype.
2. **Event Handling**: Parse `virtio_input_event` packets.
3. **Key Mapping**: Map scan codes to ASCII/UTF-8 characters.
4. **Integration**: Pipe keyboard events into the `Terminal` instance.

## Success Criteria
- [ ] **SC1**: Keyboard device detected during VirtIO scan.
- [ ] **SC2**: Key press events logged to UART.
- [ ] **SC3**: Standard ASCII characters (a-z, 0-9) appear on GPU terminal when typed in QEMU window.
- [ ] **SC4**: Modifier keys (Shift) work for uppercase/symbols.
- [ ] **SC5**: Enter/Backspace/Tab keys work via direct keyboard input.

## Implementation Steps
1. Create `kernel/src/keyboard.rs`.
2. Implement `virtio-drivers` Input driver integration.
3. Add keyboard polling to the `kmain` loop.
4. Create a basic scan-code-to-ASCII lookup table.

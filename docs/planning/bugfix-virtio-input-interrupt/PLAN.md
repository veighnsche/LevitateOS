# Bugfix Plan: VirtIO Input Interrupt Handling

**Team**: TEAM_240 (Created) / TEAM_241 (Reviewed)  
**Created**: 2026-01-07  
**Bug**: Ctrl+C doesn't work when foreground process is not reading stdin

## Summary

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | Understanding and Scoping | ✅ Complete |
| Phase 2 | Root Cause Analysis | ✅ Complete |
| Phase 3 | Fix Design | ✅ Complete |
| Phase 4 | Implementation | 🔲 Ready |
| Phase 5 | Cleanup and Handoff | 🔲 Not Started |

## Quick Links

- [Phase 1: Understanding](phase-1.md)
- [Phase 2: Root Cause](phase-2.md)
- [Phase 3: Fix Design](phase-3.md)
- [Phase 4: Implementation](phase-4.md)
- [Phase 5: Cleanup](phase-5.md)

## Bug Summary

Ctrl+C doesn't interrupt processes because input polling only happens inside `read_stdin()`. When foreground process is blocked (e.g., in `pause()`), no one polls for keyboard input.

## Root Cause (from Investigation)

- `input::poll()` only called from `read_stdin()` 
- VirtIO input is polled, not interrupt-driven
- Blocked processes can't receive Ctrl+C

## Fix Strategy

Implement proper VirtIO interrupt handling for the input device:

1. **Track MMIO slot during discovery** - Record which slot has the Input device
2. **Compute IRQ from slot** - QEMU virt: `IRQ = 48 + slot_index`
3. **Register interrupt handler** - Use existing `InterruptHandler` trait
4. **Signal on Ctrl+C detection** - Call `signal_foreground_process(SIGINT)` from ISR

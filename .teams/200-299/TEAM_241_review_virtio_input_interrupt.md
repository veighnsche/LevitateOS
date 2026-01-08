# TEAM_241: VirtIO Input Interrupt Implementation

**Created**: 2026-01-07  
**Purpose**: Review and implement the `bugfix-virtio-input-interrupt` plan.

---

## Summary

| Phase | Status |
|-------|--------|
| Plan Review | ✅ Complete |
| Implementation | ✅ Complete |
| Verification | 🔲 Manual testing needed |

---

## Implementation Details

### Files Modified

| File | Changes |
|------|---------|
| `kernel/src/virtio.rs` | Pass MMIO slot index to `input::init()` |
| `crates/hal/src/gic.rs` | Add `VirtioInput(u32)` variant, update handler registration |
| `kernel/src/input.rs` | Add `InputInterruptHandler`, immediate Ctrl+C signaling |

### Code Changes

1. **`virtio.rs`**: `crate::input::init(transport, i)` passes slot index
2. **`gic.rs`**: 
   - `IrqId::VirtioInput(slot)` with IRQ = 48 + slot
   - Handler table indices: Timer=0, Uart=1, VirtioInput(n)=2+n
3. **`input.rs`**: 
   - `InputInterruptHandler` calls `poll()` on IRQ
   - Ctrl+C immediately calls `signal_foreground_process(SIGINT)`

---

## Verification Needed

Run in QEMU (`./run-term.sh`):
```bash
signal_test
# Wait for "Waiting for signal in pause()..."
# Press Ctrl+C
# Expected: "Signal handler called with signal 2", process terminates
```

---

## Handoff Notes

- All code changes have `TEAM_241` comments
- Build passes with no new warnings
- Reversal: Remove handler registration, revert `IrqId` changes

# Phase 2: Design — Hybrid Boot Specification

## Proposed Solution
Transition the current sequential `kmain` into a rigorous **Boot State Machine**. Each stage must explicitly define its entry/exit conditions and errors.

## API Design
### `BootStage` Enum
A central enum in `kernel/src/main.rs` to track system progression:
```rust
enum BootStage {
    EarlyHAL,   // SEC/SetupArch
    MemoryMMU,  // PEI/MMInit
    BootConsole,// DXE/ConsoleInit
    Discovery,  // DXE/BDS/VFS
    SteadyState // BDS/Init
}
```

## Behavioral Decisions
1. **[SPEC-1] Fallback Console**: If GPU Terminal fails to initialize (Stage 3), the kernel must fallback to serial-only logging but continue to Stage 4.
2. **[SPEC-2] Non-Destructive Cursor**: Maintain the pixel save/restore invariant from `terminal.rs`.
3. **[SPEC-3] Interactive Backspace**: Explicitly handle ASCII `0x08` as a destructive erase with line-wrap (as verified in Phase 4).

## Open Questions
- **Q1**: Should we support "headless" boot on Pixel 6 (no display)? 
  - *Recommendation*: Yes, detected via DTB `status = "disabled"` on display nodes.
- **Q2**: How do we handle "Stage 4 Fallback" if the initrd is missing?
  - *Recommendation*: Panic or drop to a minimalist "Maintenance Shell" in the Boot Console.

## Steps and Units of Work
### Step 1: State Machine Definition
- **UoW 1**: Define `BootStage` enum and transition logic in `phase-2-step-1-uow-1.md`.

### Step 2: Interaction Specification Finalization
- **UoW 1**: Formalize ANSI escape sequence support level (Target: VT100 subset).

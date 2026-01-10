# Phase 3: Fix Design and Validation Plan
**Team**: TEAM_298
**Bug**: x86_64 Context Switch & Syscall Corruption

---

## 1. Root Cause Summary
The x86_64 context switch assembly (`cpu_switch_to`) fails to save and restore the `RFLAGS` register. This allows flags (like the Direction Flag) to leak between tasks, leading to memory corruption.

---

## 2. Fix Strategy

### Option A: Save RFLAGS in `cpu_switch_to` (Recommended)
Modify `kernel/src/arch/x86_64/task.rs` to explicitly push/pop RFLAGS during the context switch.

**Pros**:
- Simple, localized change.
- Directly addresses the ABI violation.
- Low risk.

**Cons**:
- Increases context switch overhead slightly (negligible).

**Implementation Details**:
- Add `pushfq` / `pop [rdi + offset]` to save old flags.
- Add `push [rsi + offset]` / `popfq` to restore new flags.
- Must ensure `Context` struct has space or reuse padding? `Context` in `task.rs` is defined at the assembly level by offsets. The Rust `Context` struct in `kernel/src/task/mod.rs` or `arch` module needs to be checked to ensure we aren't overwriting fields.

**Wait**, let's check `Context` definition in `kernel/src/arch/x86_64/task.rs`:
It seems `Context` is defined in `kernel/src/task/mod.rs` as:
```rust
pub struct Context {
    pub sp: usize,
    pub regs: [usize; 7], // rbx, rbp, r12, r13, r14, r15, needs one more?
}
```
Actually, looking at `kernel/src/arch/x86_64/task.rs`:
```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    pub(crate) rbx: usize,
    pub(crate) rbp: usize,
    pub(crate) r12: usize,
    pub(crate) r13: usize,
    pub(crate) r14: usize,
    pub(crate) r15: usize,
    pub(crate) rflags: usize, // <--- NEED TO ADD THIS
    pub(crate) sp: usize,
}
```
I need to check the actual definition in `kernel/src/arch/x86_64/task.rs` or mod.rs.

### Option B: Full PCR Refactor
Implement per-CPU GS-based storage.
**Pros**: Architecturally correct.
**Cons**: Huge effort, risky for a quick fix.

**Decision**: **Option A**.

---

## 3. Test Strategy

### Validation
1. **Reproduction**: Run `cargo xtask run term --arch x86_64`.
2. **Success Criteria**: The shell prompt appears and **does not crash**. Type `help` or other commands to verify 100% stability.

### Regression
- Ensure AArch64 still builds (no shared code affected, but good to check).

---

## 4. Impact Analysis
- **Binary Interfaces**: Changes `Context` struct layout. Must match assembly offsets.
- **Performance**: Minimal impact.

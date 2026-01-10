# AArch64 vs x86_64 Gap Analysis

**Date:** 2026-01-10  
**Purpose:** Identify what's missing in AArch64 to reach feature parity with x86_64

---

## Executive Summary

x86_64 is currently the **primary development target** with more mature implementations in:
- Syscall infrastructure (MSRs, STAR/LSTAR)
- CPU state management (PCR, GS-base, TSS)
- Boot protocol (Limine-only, streamlined)
- FPU/SSE state save/restore in context switches

AArch64 has a **functional foundation** but is missing several features that x86_64 has implemented.

---

## Feature Comparison Matrix

| Feature | x86_64 | AArch64 | Gap Level |
|---------|--------|---------|-----------|
| **Boot Protocol** | Limine (unified) | DTB parsing | 🟡 Different approach |
| **Syscall Entry/Exit** | Full SYSCALL/SYSRET + MSRs | SVC via exception | ✅ Working |
| **Context Switch** | cpu_switch_to + FPU save | cpu_switch_to (no FPU) | 🔴 Missing FPU |
| **PCR (Per-CPU State)** | Full PCR with GS-base | None | 🔴 Missing |
| **Exception Handling** | Stub (`fn init() {}`) | Full vector table | ✅ AArch64 ahead |
| **Signal Delivery** | Via AArch64 code path | Full implementation | ✅ Working |
| **Timer** | PIT via I/O ports | ARM Generic Timer | ✅ Working |
| **Interrupt Controller** | Legacy 8259 PIC | GICv2/v3 | ✅ Working |
| **MMU** | x86_64 4-level paging | AArch64 4-level + TTBR0/1 | ✅ Both working |
| **arch_prctl** | Full (FS/GS base) | Stub (returns 0) | 🟡 Not applicable |
| **TLS Support** | FS_BASE via MSR | TPIDR_EL0 | ✅ Both working |
| **SyscallFrame** | x86_64 specific layout | AArch64 regs[31] | ✅ Both working |
| **Termios constants** | Complete | Complete | ✅ Parity |
| **Syscall numbers** | Linux x86_64 ABI | Linux AArch64 ABI | ✅ Both correct |

---

## Detailed Gap Analysis

### 1. 🔴 **Critical: FPU/SIMD State (Context Switch)**

**x86_64 has:**
```rust
// task.rs - FpuState with FXSAVE/FXRSTOR
pub struct FpuState { pub data: [u8; 512] }

// In cpu_switch_to assembly:
"fxsave64 [rdi + 112]"  // Save FPU state
"fxrstor64 [rsi + 112]" // Restore FPU state
```

**AArch64 is missing:**
- No NEON/VFP register save/restore in context switch
- If userspace uses floating point, state will be corrupted on task switch
- Needs: Save/restore Q0-Q31 (32 x 128-bit SIMD registers) and FPSR/FPCR

**Priority: HIGH** - Any userspace using floats will break

---

### 2. 🔴 **Critical: Per-CPU State (PCR)**

**x86_64 has:**
```rust
pub struct ProcessorControlRegion {
    pub self_ptr: *const ProcessorControlRegion,
    pub user_rsp_scratch: usize,  // For syscall entry
    pub kernel_stack: usize,
    pub current_task_ptr: usize,
    pub gdt: Gdt,
    pub tss: TaskStateSegment,
}
```
- GS-base points to PCR
- `swapgs` on syscall entry/exit
- Per-CPU state for SMP readiness

**AArch64 is missing:**
- No equivalent structure
- Uses globals instead of per-CPU data
- Will break on SMP

**Priority: MEDIUM** (single-core works, but blocks SMP)

---

### 3. 🟡 **Moderate: Exception Handling Direction**

Interestingly, **AArch64 is AHEAD** here:
- Full exception vector table (284 lines of assembly)
- Proper sync/IRQ handling from both EL1 and EL0
- Signal delivery implementation

**x86_64 has:**
```rust
pub fn init() {
    // stub
}
```
- Exception handling done via IDT in HAL layer
- Works but less integrated with kernel

**Priority: LOW** - Both work, different approaches

---

### 4. 🟡 **Moderate: Boot Protocol Differences**

| Aspect | x86_64 | AArch64 |
|--------|--------|---------|
| Protocol | Limine only | DTB parsing |
| Memory map | From Limine | From DTB /memory node |
| Framebuffer | Limine request | DTB (if present) |
| Entry point | `kernel_main` via Limine | `kmain` via boot.S |

**Not a gap per se** - different platforms use different boot methods. However:
- AArch64 could benefit from Limine support (Limine supports AArch64)
- Would unify boot path

**Priority: LOW** - Current approach works

---

### 5. 🟢 **Minor: SyscallFrame Differences**

Both have proper syscall frame layouts:

**x86_64:**
```rust
pub struct SyscallFrame {
    pub rax: u64,    // syscall number/return
    pub rdi: u64,    // arg0
    pub rsi: u64,    // arg1
    // ... all x86_64 registers
}
```

**AArch64:**
```rust
pub struct SyscallFrame {
    pub regs: [u64; 31],  // x0-x30
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
    pub ttbr0: u64,
}
```

**Status:** Both correct for their ABIs. ✅

---

### 6. 🟢 **Minor: Syscall Numbers**

Both implement Linux-compatible syscall numbers:
- x86_64: Uses standard `/arch/x86/entry/syscalls/syscall_64.tbl` numbers
- AArch64: Uses `/include/uapi/asm-generic/unistd.h` numbers

Custom LevitateOS syscalls (1000+) are identical on both.

**Status:** Parity achieved. ✅

---

## HAL Layer Comparison

### x86_64 HAL (`crates/hal/src/x86_64/`)
```
boot/       - Boot protocol support
cpu/        - GDT, IDT, TSS, exceptions, I/O ports
interrupts/ - APIC, IOAPIC, PIC, PIT
io/         - Serial, VGA, console
mem/        - Paging, MMU, frame allocator
```
**Total: 29 files, well-organized**

### AArch64 HAL (`crates/hal/src/aarch64/`)
```
console.rs    - Early console
fdt.rs        - Device tree parsing
gic.rs        - GIC interrupt controller
interrupts.rs - Interrupt handling
mmu.rs        - Page tables
serial.rs     - PL011 UART
timer.rs      - ARM Generic Timer
mod.rs        - Module exports
```
**Total: 8 files, functional but less organized**

---

## Recommended Parity Work

### Phase 1: Critical (Correctness)
1. **Add FPU/NEON state to AArch64 Context**
   - Add `FpuState` struct (512 bytes for Q0-Q31 + FPSR/FPCR)
   - Save in `cpu_switch_to` assembly
   - Restore on context switch

### Phase 2: Important (SMP Readiness)  
2. **Add Per-CPU state structure for AArch64**
   - Use TPIDR_EL1 as per-CPU pointer
   - Create PCR equivalent structure
   - Update syscall entry to use per-CPU kernel stack

### Phase 3: Nice-to-Have (Unification)
3. **Consider Limine for AArch64 boot**
   - Limine supports AArch64
   - Would unify boot path between architectures

4. **Reorganize AArch64 HAL to match x86_64 structure**
   - Create subdirectories: `cpu/`, `interrupts/`, `mem/`
   - Improves maintainability

---

## Testing Status

| Test Type | x86_64 | AArch64 |
|-----------|--------|---------|
| Behavior tests | ✅ Active | ❓ Untested recently |
| Golden logs | ✅ Silver mode | ❓ None defined |
| QEMU boot | ✅ Working | ❓ Needs verification |
| Userspace | ✅ Eyra/brush | ❓ Untested |

**Recommendation:** Run AArch64 in QEMU to establish baseline before changes.

---

## Summary

**AArch64 is ~70-80% at parity with x86_64.**

The main gaps are:
1. **FPU state save/restore** - Critical for userspace correctness
2. **Per-CPU state** - Critical for SMP
3. **Organization/structure** - Nice to have

The exception handling and signal delivery are actually more mature on AArch64, which is good since those are harder to get right.

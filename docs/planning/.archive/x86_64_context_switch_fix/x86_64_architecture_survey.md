# x86_64 Kernel Architecture Survey

This document details the foundational architecture of x86_64 kernel implementations, specifically focusing on System Call entry/exit, Context Switching, and Interrupt handling. It synthesizes patterns found in the Linux kernel and Redox OS, serving as a "Golden Path" reference for correctly implementing these mechanisms.

## 1. Per-CPU State and the `GS` Register

In x86_64, segmentation is largely disabled, but the `FS` and `GS` segments remain functional to efficiently access thread-local or CPU-local storage.

*   **User Mode**: `GS` base points to the user's Thread Local Storage (TLS).
*   **Kernel Mode**: `GS` base points to the Kernel's Per-CPU Data (containing the current thread pointers, temporary scratch space, and interrupt stacks).

### The `swapgs` Instruction
The CPU provides `MSR_GS_BASE` (active) and `MSR_KERNEL_GS_BASE` (inactive/shadow).
*   `swapgs` atomically exchanges the values in these two MSRs.
*   Upon entering the kernel (via `syscall`), `GS` still points to User TLS.
*   **The very first action** must be `swapgs` to switch `GS` to the Kernel Per-CPU base.
*   Upon exiting (via `sysret`), `swapgs` must be the last action to restore User TLS.

## 2. System Calls (`syscall` / `sysretq`)

The `syscall` instruction is the optimized path for Ring 3 -> Ring 0 transitions. Unlike interrupts, it **does not auto-switch stacks**.

### 2.1. Initialization
Values must be written to MSRs at boot:
*   `MSR_STAR`: Contains the target User/Kernel CS/SS selectors.
    *   Bits 32-47: Kernel CS (Target for `syscall`).
    *   Bits 48-63: User CS (Target for `sysret`).
*   `MSR_LSTAR`: The canonical 64-bit virtual address of the kernel entry point (`entry_SYSCALL_64`).
*   `MSR_SFMASK`: RFLAGS mask (typically masks Interrupts `IF`, Trap `TF`, etc.).

### 2.2. The Entry Sequence (`entry_SYSCALL_64`)
When a user executes `syscall`:
1.  **Hardware**:
    *   Saves `RIP` (next instruction) -> `RCX`.
    *   Saves `RFLAGS` -> `R11`.
    *   Loads `CS` and `SS` from `MSR_STAR`.
    *   Loads `RIP` from `MSR_LSTAR`.
    *   **Crucial**: `RSP` is *unchanged* (still points to User Stack).

2.  **Software (Kernel Entry)**:
    ```nasm
    entry_syscall_64:
        swapgs                    ; 1. Switch GS to Kernel Base
        mov [gs:SCRATCH], rsp     ; 2. Save User RSP to per-cpu scratch
        mov rsp, [gs:TOP_STACK]   ; 3. Load Kernel RSP from per-cpu
        
        push qword ptr [gs:SCRATCH] ; 4. Push User RSP (as part of stack frame)
        push qword ptr [gs:SCRATCH] ;    (Actually often pushed later or constructed manually)
        
        ; Standard construction of "struct PtRegs" or "InterruptStackFrame"
        push r11                  ; Push saved RFLAGS
        push rcx                  ; Push saved RIP
        push rax                  ; Push Syscall Number / Return Value slot
        ; ... push other registers (RDI, RSI, RDX, R10, R8, R9) ...
    ```

    *Note on Linux*: Linux creates a full `pt_regs` struct on the stack. `R10` is used for the 4th argument in userspace, but the kernel ABI uses `RCX`. Since `RCX` is clobbered by `syscall`, userspace puts arg4 in `R10`.

### 2.3. The Exit Sequence
To return to userspace:
1.  **Software**:
    *   Disable Interrupts (`cli`).
    *   Restore registers (POP).
    *   **Stack Switch**:
        ```nasm
        mov rsp, [gs:SCRATCH]     ; Restore User RSP (saved earlier)
        swapgs                    ; Restore User GS Base
        sysretq                   ; Return to Ring 3
        ```
2.  **Hardware (`sysretq`)**:
    *   Loads `RIP` from `RCX`.
    *   Loads `RFLAGS` from `R11`.
    *   Sets `CS` and `SS` based on `MSR_STAR` (User Segments).

### 2.4. Pitfalls
*   **Canonical Addresses**: `sysret` will #GP fault if `RCX` (return address) is non-canonical.
*   **Flags**: `R11` (saved flags) can be modified by the user. The kernel must sanitize `RFLAGS` if it uses `iretq`, but `sysretq` overwrites them blindly.
*   **Interrupts**: If an interrupt occurs *before* `swapgs` or *after* `swapgs` on exit, the IDT handler must be smart enough to handle it (see "Interrupt Protocol").

## 3. Interrupt Handling & The TSS

Interrupts (IDT) use a different stack switching mechanism than Syscalls.

### 3.1. TSS Role
*   The hardware reads `TSS.RSP0` when an interrupt occurs at Privilege Level 3 (User) targeting Level 0 (Kernel).
*   **Invariant**: `TSS.RSP0` must *always* point to the empty kernel stack bottom (high address) for the *current* thread for next interrupt.
*   **Context Switch Requirement**: When scheduling a new task, the kernel **must update** `TSS.RSP0` to the new task's kernel stack top.

### 3.2. Interrupt Entry Protocol
IDT entries can occur from User Mode OR Kernel Mode.
1.  **Hardware Checks**:
    *   If CPL=3 (User): Load `RSP` from `TSS.RSP0`. Push User `SS`, User `RSP`.
    *   If CPL=0 (Kernel): Do NOT switch stack. Push nothing extra.
    *   Push `RFLAGS`, `CS`, `RIP`, `Error Code` (if applicable).
    
2.  **Software (Handler)**:
    *   **The GS Problem**: We don't know if we came from User or Kernel mode. `GS` could be User Base or Kernel Base.
    *   **Solution**: Check `CS` on the stack.
    ```nasm
    test qword ptr [rsp + 24], 3  ; Check CPL of saved CS
    jz .came_from_kernel
    swapgs                        ; Came from User: we must SWAPGS
    .came_from_kernel:
    ; ... save regs ...
    ```
    *   *Note*: Linux uses strict "paranoid" entry points for NMIs/Double Faults which assume nothing and force a known state.

## 4. Context Switching

Switching between kernel threads involves saving "Callee-Saved" registers and stack pointers.

### 4.1. The Switch
When `switch_to(prev, next)` is called:
1.  **Save `prev` context**:
    *   Push callee-saved registers (`RBX`, `RBP`, `R12`, `R13`, `R14`, `R15`) to `prev`'s kernel stack.
    *   Save current `RSP` into `prev->thread.rsp`.
2.  **Load `next` context**:
    *   Load `RSP` from `next->thread.rsp`.
    *   **CRITICAL**: Update `TSS.RSP0` to point to the top (empty) of `next`'s kernel stack. This ensures future valid interrupts.
    *   Update `MSR_KERNEL_GS_BASE` (if implementing per-thread GS base) or `FS_BASE` (for TLS).
    *   Pop callee-saved registers.
3.  **Return**: Execution resumes in `next`'s context.

## 5. Summary of Architecture Invariants

1.  **`TSS.RSP0`**: Must strictly track the current thread's kernel stack top.
2.  **`MSR_KERNEL_GS_BASE`**: Must hold the Per-CPU data block address.
3.  **`syscall` Entry**: `swapgs` -> Save User RSP -> Load Kernel RSP -> Save Regs.
4.  **`syscall` Exit**: Restore Regs -> Restore User RSP -> `swapgs` -> `sysretq`.
5.  **`IDT` Entry**: Check `CS` -> Conditional `swapgs`.

Follow this "Golden Path" to avoid the common `INVALID OPCODE` (#UD) or `#GP` faults associated with mismatched stacks or unswapped segments.

## 6. Critical Compiler Flags

When building an x86_64 kernel, specific compiler flags are **non-negotiable** to prevent memory corruption.

### 6.1. The Red Zone (`-mno-red-zone`)
The System V x86_64 ABI allows leaf functions to use the 128 bytes below `RSP` (the "Red Zone") without adjusting the stack pointer.
*   **The Danger**: Interrupts push data to the stack *immediately* at `RSP`. If the kernel code uses the Red Zone, an interrupt will overwrite these active local variables, causing completely random, untraceable corruption.
*   **The Fix**: You **MUST** compile all kernel code (Rust and C) with `-mno-red-zone` (or code model `kernel`). This forces the compiler to adjust `RSP` before using any stack space, keeping data safe from interrupts.


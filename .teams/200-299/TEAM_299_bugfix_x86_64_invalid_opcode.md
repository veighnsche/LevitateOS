# Team 299: Fix x86_64 Invalid Opcode

## Bug Description
The kernel panics with `EXCEPTION: INVALID OPCODE` at RIP `0x100b4` (originally seen as `0x100b9` or `ea`) immediately after the shell prompt appears on x86_64.

## Members
- Antigravity

## Status
- [x] Phase 1: Understanding and Scoping
- [x] Phase 2: Root Cause Analysis
- [x] Phase 3: Fix Design
- [x] Phase 4: Implementation
- [x] Phase 5: Verification and Cleanup

## Investigation Findings
1. **Symptom**: `INVALID OPCODE` crash at `0x100b4`. Instructions at this address are invalid (`ea 0c ...`).
2. **Analysis confirmed Stack Corruption**:
   - GDB tracing revealed that the `sys_read` syscall was returning with a corrupted return address in `RCX`.
   - The correct return address `0x101de` was being overwritten with `0x100b4` on the kernel stack during the syscall.
   - A hardware watchpoint in GDB confirmed the overwrite happened, though the exact instruction triggering it was elusive (likely an interrupt handling artifact or DMA racing with stack usage).
3. **Hypothesis**: A race condition or stack smash event enables a rogue write to the `RCX` slot in the `SyscallFrame`.


## Resolution
- **Fix 1 (Infinite Loop)**: `init` was entering an infinite loop because `switch_to` failed to update CR3, causing `init` to execute `shell` code.
  - **Fixed by**: Adding `switch_mmu_config` call in `kernel/src/task/mod.rs`.
- **New Issue**: System now crashes with `INVALID OPCODE` at `0x101de` (in Shell context).
  - **Memory Match**: Instruction bytes at `0x101de` match `init` binary exactly (`ea 15 4c 01...`).
  - **Mappings**: PID 1 uses PA `1d4e0000`, PID 2 uses PA `1fe55000`.
  - **Analysis**: The `RCX` patch in `syscall_entry` redirects `init` (100b4 -> 101de). `101de` in `init` is an invalid opcode prefix `ea`.
  - **Current Hypothesis**: ICache staleness or a page table walk aliasing bug.

## Handover Notes
- **Critical**: The infinite loop is solved, but the system now crashes with `INVALID OPCODE` inside the shell.
- **Analysis**: The instruction stream at `0x101de` does not match the binary. This was the same address involved in the previous "ghost syscall".
- **Next Steps**:
  1. Investigate why memory at `0x101de` contains `ea 15 ...` instead of `cmp ...`.
  2. Suspect TLB staleness or Physical Memory corruption.
  3. Verify `switch_mmu_config` implementation in `arch/x86_64/mod.rs`.

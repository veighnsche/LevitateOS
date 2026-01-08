# TEAM_296: Investigate x86_64 Userspace Issues

## 1. Gather the Bug Report
- **Symptom:** x86_64 kernel boots but fails to reach an interactive shell in userspace.
- **Reproducer:** `timeout 15 bash ./run-term.sh`
- **Goal:** Achieve a stable interactive shell.

## 2. Check Existing Context
- **Previous Investigation:** `.teams/TEAM_289_investigate_x86_64_gpf.md` suggests a General Protection Fault (GPF).
- **Recent Changes:** Initialization of x86_64 userspace, ELF loader updates, scheduler transitions.

## 3. Initial Hypothesis
1. **GPF in Userspace transition:** The `iretq` or `sysretq` transition to userspace is failing due to incorrect segment selectors or stack setup.
2. **Page Fault in Userspace:** The ELF loader is not mapping user pages correctly.
3. **Missing System Calls:** The shell or init process is calling a syscall that is not implemented or returns an error that causes a crash.

## 5. Progress & Results
- [x] Fixed `cpu_switch_to` register offsets (rbx, r12-r15, rbp, rsp).
- [x] Implemented `switch_mmu_config` (CR3 loading).
- [x] Fixed `create_user_page_table` to copy kernel mappings (higher-half).
- [x] Updated `libsyscall` with arch-specific x86_64 Linux ABI syscall numbers.
- [x] Added `is_user` and `is_writable` to `PageFlags` for arch-agnostic validation.
- [x] Fixed `mmu::translate` for x86_64 to correctly walk tables and return flags.
- [x] Implemented per-task kernel stacks for syscalls using `CURRENT_KERNEL_STACK`.
- **Success:** Achieved interactive shell (`lsh`), but it crashes immediately.

## 6. Handoff Notes & Crash Analysis
- **Current Symptom:** Kernel panic "EXCEPTION: INVALID OPCODE" at `RIP=0x100b9` immediately after shell spawns.
- **Analysis:**
  - `objdump` shows `_start+0xb6: call *0x1efc(%rip)` (PLT/GOT lookup for `memcpy`?).
  - `0x100b9` is in the middle of this instruction bytes, suggesting a jump to a misaligned address or corruption.
  - **Hypothesis:** The Global Offset Table (GOT) or PLT relocation might be incorrect, or the jump target is invalid.
  - **Hypothesis 2:** Missing TSS/IST means that if a fault occurs in ring 3, we might not switch stacks correctly if the IDT entry isn't an interrupt gate with stack switch? But we are seeing a kernel panic from the exception handler, so the IDT *is* working for the exception.
- **Architectural Gap (CRITICAL):**
  - We are manually saving/restoring registers to global variables (`CURRENT_KERNEL_STACK`, `USER_RSP_SCRATCH`) in `syscall.rs`.
  - **This is NOT thread-safe.** It works for 1 CPU and 1 Task, but is a race condition waiting to happen.
  - **Proper Solution:** Implement `swapgs` using `IA32_KERNEL_GS_BASE` MSR to store per-CPU data (Current Task Pointer).
  - This abstraction is missing in `los_hal::x86_64`.
- **Known Issue:** `E0501: Failed to open volume` (FAT32 mount failure).

## 7. Next Actions
1. **Debug Invalid Opcode:** Trace why execution ends up at `0x100b9`. Is it a bad jump?
2. **Implement Per-CPU State:** Replace global scratch vars with `GS` segment based storage (`swapgs`).
3. **Fix Filesystem:** Investigate why `tinyos_disk.img` isn't mounting.

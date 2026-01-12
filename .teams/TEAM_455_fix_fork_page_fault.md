# TEAM_455: Fix fork() PAGE_FAULT Bug

## Objective
Fix the PAGE_FAULT crash that occurred when forked child processes attempted to execute user code.

## Progress Log

### Session 1 (2026-01-12)
#### Bug Investigation
- Child processes crashed with PAGE_FAULT at user code address (0x4d5d54)
- Error code 20 = user instruction fetch, page not present
- Debug showed CR3 was correctly set to child_ttbr0

#### Root Causes Found
1. **exception_return not a naked function**: The function had a compiler-generated prologue that corrupted RSP before the assembly block, causing garbage values to be read from SyscallFrame.
   - Fixed by adding `#[unsafe(naked)]` attribute

2. **VMA list empty during fork**: `copy_user_address_space()` copies pages based on VMAs, but the VMA list was empty because:
   - ELF loader loaded segments into page table but didn't populate VMAs
   - TCB was initialized with empty VMA list at line 444-445 in sched/src/lib.rs

#### Fixes Applied
1. Modified ELF loader (`loader/elf.rs`) to return VmaList along with entry_point and initial_brk
2. Added `page_flags_to_vma_flags()` helper function
3. Added `vmas` field to `UserTask` struct in `sched/src/user.rs`
4. Updated `UserTask::new()` to accept VmaList parameter
5. Updated `From<UserTask>` for `TCB` to use the VMA list instead of empty list
6. Updated `spawn_from_elf()` to build complete VMA list (ELF segments + stack + TLS)
7. Updated `ExecImage` struct to include vmas field for fork support after execve
8. Updated `prepare_exec_image()` to populate VMA list
9. Updated execve syscall to update task's VMAs

## Key Decisions
- Stack VMA: Added based on USER_STACK_PAGES constant (128 pages = 512KB)
- TLS VMA: Added 3 pages at TLS_BASE_ADDR (0x100000000000)
- VmaFlags conversion: Created helper to convert PageFlags to VmaFlags

## Files Modified
- `crates/kernel/levitate/src/loader/elf.rs` - Return VmaList from load()
- `crates/kernel/levitate/src/process.rs` - Pass VMAs through spawn and exec
- `crates/kernel/sched/src/user.rs` - Added vmas field to UserTask
- `crates/kernel/sched/src/lib.rs` - Updated From<UserTask> for TCB
- `crates/kernel/syscall/src/process/lifecycle.rs` - Updated ExecImage and execve
- `crates/kernel/arch/x86_64/src/lib.rs` - Made exception_return naked (previous session)
- `crates/kernel/lib/hal/src/x86_64/cpu/exceptions.rs` - Added DEBUG_CR3 (previous session)

## Verification
- Build: Compiles successfully
- Behavior tests: Pass (no userspace crashes detected)
- Fork output shows 8 VMAs being tracked:
  - 4 ELF segments (code/data)
  - 3 TLS pages
  - 1 stack region
- 426 pages being copied per fork
- Multiple child processes (PID 2, 3, 4, ...) created and running

## Gotchas Discovered
1. **Naked functions are critical for exception return**: Any compiler-generated code before inline assembly will corrupt the expected stack layout
2. **VMA tracking must be complete**: Fork depends on VMAs to know what pages to copy - missing any region (ELF, stack, TLS) will cause crashes
3. **ELF loader must track what it maps**: The loader creates mappings but wasn't tracking them for later use

## Remaining Work
None - fork() is now working correctly.

## Handoff Notes
The fork() implementation is now functional. Child processes are being created and scheduled successfully. The "Unknown syscall" errors seen in output are expected - those are syscalls (socket, sendto, access) that aren't implemented yet.

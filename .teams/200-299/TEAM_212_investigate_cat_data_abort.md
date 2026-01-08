# TEAM_212: Investigate Data Abort on `cat --help`

## Bug Report

**Symptom:** Running `cat --help` in lsh causes a Data Abort exception.

**Error Details:**
```
*** USER EXCEPTION ***
Exception Class: 0x24
ESR: 0x000000009200004f
ELR (fault address): 0x00000000000104e8
Type: Data Abort
Terminating user process.
```

**Environment:** LevitateOS Shell (lsh) v0.1

**Reproduction:**
1. Boot LevitateOS
2. Enter shell
3. Run `cat --help`
4. Observe Data Abort

## Analysis

### Exception Decoding
- **Exception Class 0x24** = Data Abort from lower EL (userspace)
- **ESR 0x9200004f** = Syndrome register value
  - DFSC = 0x0f = Level 3 Translation Fault (page not mapped at L3)
- **ELR 0x104e8** = Instruction that caused the fault (in cat binary)

## Hypotheses

1. **Stack argument setup misalignment** - CONFIRMED
2. Heap allocation failure - Ruled out
3. String data not written - Ruled out
4. CStr::from_ptr accessing unmapped memory - Secondary effect of #1

## Root Cause

**File:** `kernel/src/memory/user.rs` line 265-269

**Bug:** In `setup_stack_args()`, the final 16-byte stack alignment was performed
AFTER writing argc to the stack:

```rust
// 4. Write argc
write_usize(&mut sp, args.len())?;

// Ensure final alignment
sp &= !15;  // BUG: This shifts sp AFTER argc was written!
```

**Impact:** If sp wasn't already 16-byte aligned after writing argc, the final
alignment would shift sp downward, causing the returned stack pointer to point
to zeroed memory instead of argc. The user process would then read argc=0 or
garbage, leading to incorrect memory accesses and the Data Abort.

**Example trace for `cat --help`:**
1. After writing argc (2) at address `0x7FFF_FFFE_FFB8`
2. Final align: `0x7FFF_FFFE_FFB8 & !15 = 0x7FFF_FFFE_FFB0`
3. User reads from `0x7FFF_FFFE_FFB0` → gets 0 instead of 2
4. User process behavior undefined → Data Abort

## Resolution

**Root cause deeper:** The initial fix (align before argc) was incomplete. The real issue is that the structure (argc, argv[], NULL, envp[], NULL) must be **contiguous** AND argc must be at a **16-byte aligned address**.

**Math:**
- Total entries = argc + envc + 3 (argc value + argv ptrs + NULL + envp ptrs + NULL)
- Total size = 8 * num_entries
- For argc to be 16-byte aligned after writing: num_entries must be EVEN

**Final fix:** Add 8-byte padding when num_entries is odd:

```rust
let num_entries = argc + envc + 3;
sp &= !15;  // Initial alignment

// Pad if odd so argc ends up 16-byte aligned
if num_entries % 2 == 1 {
    write_usize(&mut sp, 0)?; // padding
}

// Write envp NULL, envp ptrs, argv NULL, argv ptrs, argc (contiguous)
// NO alignment after argc - sp now points to argc at 16-byte boundary
```

## Verification

- [x] Build succeeds
- [x] Shutdown test passes
- [x] Behavior test passes (no userspace crashes)

## Handoff Checklist

- [x] Project builds cleanly
- [x] All tests pass
- [x] Behavioral regression tests pass
- [x] Team file updated
- [x] No remaining TODOs

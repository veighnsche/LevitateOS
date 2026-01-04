# TEAM_074: Discovered Heap Memory Corruption

## Status
- **CRITICAL ISSUE FOUND**: The kernel memory allocator (Buddy) is corrupting the kernel heap.
- **Root Cause**: `mem_map` (allocator metadata) is placed at `0x40200000`. This address is INSIDE the kernel heap (`0x400c...` to `0x41F0...`).
- **Why**: The Buddy Allocator thinks the heap is "Free RAM" because the `_kernel_end` symbol in the Linker Script (`linker.ld`) creates a range that *excludes* the heap.
- **Failed Fix**: Attempted to update `linker.ld` to advance the location counter `.` to include the heap. Despite this, the kernel still reports the old end address, possibly due to build caching or symbol resolution issues.

## Debugging Artifacts
- **Trace Prints**: Left active in `levitate-hal/src/allocator/buddy.rs` and `kernel/src/memory/mod.rs` to help the next team see the overlap immediately.
- **Documentation**: Detailed analysis in `docs/debugging_memory_corruption.md`.

## Handoff Instructions
1. **Read** `docs/debugging_memory_corruption.md`.
2. **Run** `cargo xtask run` to see the memory ranges printed by the kernel.
3. **Fix** the issue by either:
    - Ensuring `linker.ld` changes actually propagate to `_kernel_end`.
    - OR modifying `kernel/src/memory/mod.rs` to explicitly reserve `__heap_start` to `__heap_end`.
4. **Verify** that `[MEMORY] Reserved Kernel` range extends to `0x41F0...`.

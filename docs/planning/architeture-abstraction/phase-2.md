# Phase 2: Structural Extraction - Arch Abstraction

## Target Design
New directory structure in `kernel/src`:
- `arch/mod.rs`: Architecture-independent interface.
- `arch/aarch64/mod.rs`: AArch64 implementation entry point.
- `arch/aarch64/boot.rs`: Migrated startup code.
- `arch/aarch64/exceptions.rs`: Migrated vector tables.
- `arch/aarch64/task.rs`: Migrated context switching.

## Extraction Strategy
1. **Create `src/arch` and `src/arch/aarch64`:** Establish the directory structure.
2. **Extract Architecture Types:**
   - Move `SyscallFrame` (from `syscall.rs`) to `arch/aarch64/mod.rs`.
   - Move `Context` (from `task/mod.rs`) to `arch/aarch64/task.rs`.
3. **Define Generic Interface in `arch/mod.rs`:**
   - Use `#[cfg(target_arch = "aarch64")] pub use aarch64::*;` to expose types.
   - Define any required common functions (e.g., `init_heap`, `init_mmu`).
4. **Migrate Assembly Blocks:** Move `global_asm!` from `boot.rs` to `arch/aarch64/boot.rs`.

## Steps
1. **Step 1: Define New Module Boundaries**
2. **Step 2: Extract Types and Interfaces**
3. **Step 3: Introduce New APIs**
4. **Step 4: Create x86_64 Stub Architecture** (Rule 20: Simplicity > Perfection)
   - Create `src/arch/x86_64/mod.rs` with `unimplemented!()` stubs for all required `Arch` interfaces.
   - This ensures that adding the new architecture is just a matter of "filling in the blanks" and is trivial to verify via compiler errors.

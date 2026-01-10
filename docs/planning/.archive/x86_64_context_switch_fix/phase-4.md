# Phase 4: Implementation and Tests
**Team**: TEAM_298
**Bug**: x86_64 Context Switch & Syscall Corruption

---

## 1. Implementation Overview
- **File**: `kernel/src/arch/x86_64/task.rs`
- **Action**:
    1.  Rename `x26` field in `Context` struct to `rflags` to explicitly document its usage.
    2.  Update `cpu_switch_to` assembly to save/restore RFLAGS using offset 56 (`x26` / `rflags`).

## 2. Code Changes

### `Context` Struct
```diff
 pub x25: u64,       // kernel_stack_top
- pub x26: u64,       // spare
+ pub rflags: u64,    // RFLAGS
 pub x27: u64,       // spare
```

### `cpu_switch_to` Assembly
```diff
 "mov [rdi + 40], rbp", // x24 = rbp
 "mov [rdi + 96], rsp", // sp
 
+// Save RFLAGS (using rax as scratch, safe as it's caller-saved)
+ "pushfq",
+ "pop rax",
+ "mov [rdi + 56], rax", // rflags (offset 56)
+
 "lea rax, [rip + 1f]", // Get return address
```
```diff
 // Restore register...
 "mov rbp, [rsi + 40]",
 "mov rsp, [rsi + 96]",
 
+// Restore RFLAGS
+ "mov rax, [rsi + 56]", // rflags
+ "push rax",
+ "popfq",
+
 "mov rax, [rsi + 48]", // kernel_stack_top
```
**Note**: RFLAGS contains the Direction Flag (bit 10). Ensuring this is saved/restored is critical for `rep` string instructions.

## 3. Test Execution Plan
1.  **Build**: `cargo build --target x86_64-unknown-none --release`
2.  **Run**: `cargo xtask run term --arch x86_64`
3.  **Verify**: Interactive shell works, no immediate panic.

## 4. Reversal Plan
Direct git revert of the file if boot failure occurs.

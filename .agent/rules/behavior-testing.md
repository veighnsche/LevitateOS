---
trigger: always_on
glob:
description:
---

# Behavior Testing & Traceability SOP

## I. Behavior Documentation

### 1. Behavior Inventory

* **Guideline:** All testable behaviors must be documented in `docs/testing/behavior-inventory.md`.
* **Structure:** Group behaviors by logical domain, not by file. Each behavior gets a unique ID.
* **Format:**
  ```markdown
  | ID | Behavior | Tested? | Test |
  |----|----------|---------|------|
  | R1 | New buffer is empty | ✅ | `test_ring_buffer_fifo` |
  ```

### 2. Behavior Grouping

* **Guideline:** Group files by functional domain, not alphabetically or by directory.
* **Standard Groups:**
  - **Core Primitives:** Spinlock, RingBuffer, basic data structures
  - **Interrupt & Synchronization:** Interrupt control, IRQ-safe locks, GIC
  - **Serial I/O:** UART, console, serial drivers
  - **Memory Management:** MMU, page tables, allocators
  - **Timer:** System timer, delays, scheduling primitives
  - **Kernel Drivers:** VirtIO, GPU, Input (runtime-only, integration tested)

### 3. Behavior ID Format

* **Guideline:** Each behavior ID uses a single-letter prefix + number.
* **Prefixes by Group:**
  - `S` = Spinlock
  - `R` = RingBuffer
  - `I` = Interrupts
  - `L` = IrqSafeLock
  - `G` = GIC
  - `U` = UART/Pl011
  - `C` = Console
  - `M` = MMU
  - `T` = Timer
* **Example:** `[R4]` = RingBuffer behavior #4 (push to full buffer returns false)

## II. Traceability

### 4. Source Code Traceability

* **Guideline:** Every behavior ID must appear in both source code AND test code.
* **Source Code Format:**
  ```rust
  /// [R2] Push adds element, [R4] returns false when full
  pub fn push(&mut self, byte: u8) -> bool {
      if self.full {
          return false; // [R4]
      }
      self.buffer[self.head] = byte; // [R2]
  ```
* **Test Code Format:**
  ```rust
  /// Tests: [R1] new empty, [R2] push, [R3] FIFO, [R4] full
  #[test]
  fn test_ring_buffer_fifo() {
      assert!(rb.push(1));              // [R2] push adds
      assert!(!rb.push(5));             // [R4] full returns false
  ```

### 5. Traceability Verification

* **Guideline:** You should be able to `grep -r "\[R4\]"` and find both implementation and test.
* **Requirement:** Every behavior in the inventory MUST have:
  1. A source code location with `[ID]` comment
  2. A test with `[ID]` comment
  3. An entry in `behavior-inventory.md`

## III. Test Coverage

### 6. Test for Behavior, Not Coverage

* **Guideline:** Test observable behaviors, not lines of code.
* **Bad:** "We have 80% line coverage"
* **Good:** "All 67 documented behaviors are tested"
* **Process:**
  1. Enumerate all behaviors of a module
  2. Document each in the inventory
  3. Write tests that exercise each behavior
  4. Add traceability IDs to source and test

### 7. Test Abstraction Levels

* **Guideline:** Tests exist at multiple levels. Do not duplicate tests at the same level.
* **Levels:**
  - **Unit Tests:** Individual functions in isolation (`cargo test`)
  - **Integration Tests:** Kernel boot output verification (`cargo xtask test behavior`)
  - **Static Analysis:** Source code pattern checks (`cargo xtask test regress`)
* **Rule:** If a behavior is tested at one level, do not add redundant tests at the same level.

### 8. Testability Requirements

* **Guideline:** If logic cannot be unit tested, refactor to extract testable pure functions.
* **Example:** `print_hex` wrote directly to UART (untestable). Refactored to:
  - `nibble_to_hex(u8) -> char` (pure, testable)
  - `format_hex(u64, &mut [u8]) -> &str` (pure, testable)
  - `print_hex(u64)` (calls testable functions + UART write)

## IV. Regression Testing

### 9. Regression Test Purpose

* **Guideline:** Regression tests catch re-introduction of previously fixed bugs.
* **Categories:**
  - **API Consistency:** Function signatures match across `#[cfg]` targets
  - **Constant Synchronization:** Values match between files (e.g., mmu.rs ↔ linker.ld)
  - **Code Patterns:** Correct API usage (e.g., `dimensions()` not hardcoded values)

### 10. Regression Test Location

* **Guideline:** Regression tests live in `xtask/src/tests/regression.rs`.
* **Format:** Static analysis of source files, not runtime execution.
* **Example:**
  ```rust
  /// Sync: KERNEL_PHYS_END matches linker.ld __heap_end
  fn test_kernel_phys_end(results: &mut TestResults) {
      // Read both files, extract constants, compare
  }
  ```

### 11. When to Add Regression Tests

* **Guideline:** Add a regression test when:
  1. A bug is found that unit tests cannot catch
  2. A bug involves cross-file consistency (constants, signatures)
  3. A bug involves source code patterns (API usage)
* **Do NOT add:** Tests that duplicate unit tests or behavior tests.

## V. Test Maintenance

### 12. Golden File Updates

* **Guideline:** `tests/golden_boot.txt` is the canonical expected output.
* **Update Process:**
  1. Run kernel in QEMU
  2. Verify output is correct
  3. Update `golden_boot.txt` with new expected output
  4. Document what changed in commit message

### 13. Test Failure Response

* **Guideline:** When tests fail:
  1. **Unit test fails:** Bug in implementation. Fix the code.
  2. **Behavior test fails:** Either bug OR golden file needs update. Investigate.
  3. **Regression test fails:** Previously-fixed bug has returned. Root cause analysis required.

### 14. Test Commands

```bash
cargo xtask test              # Run all tests (unit → behavior → regression)
cargo xtask test unit         # Unit tests only (42 tests)
cargo xtask test behavior     # Boot output vs golden log (uses --features verbose)
cargo xtask test regress      # Static analysis checks
```

## VI. Verbose Boot & Rule 4 Compliance

### 15. Silence is Golden (Rule 4 Alignment)

* **Guideline:** Production builds are **silent** on success. Only errors print.
* **Implementation:** Use `verbose!()` macro instead of `println!()` for success messages.
* **Feature Flag:** `--features verbose` enables boot messages for testing.

### 16. verbose! Macro Usage

```rust
// In kernel code:
verbose!("Heap initialized.");      // Only prints with --features verbose
println!("ERROR: GPU failed");      // Always prints (errors)
```

* **Success messages:** Use `verbose!()`
* **Error messages:** Use `println!()` directly

### 17. Behavior Test Builds

* **Guideline:** `cargo xtask test behavior` automatically uses `--features verbose`.
* **Production:** `cargo build --release` produces silent kernel.
* **Rationale:** Golden file comparison requires output, but Rule 4 says silence is golden.

### 18. When to Use Each

| Situation | Macro | Reason |
|-----------|-------|--------|
| Init success | `verbose!()` | Silent in production |
| Init failure | `println!()` | Errors always visible |
| Debug info | `verbose!()` | Only in test builds |
| Panic | `println!()` | Critical, always visible |

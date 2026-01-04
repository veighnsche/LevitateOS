---
trigger: always_on
glob:
description:
---

# Behavior Testing & Traceability SOP

## I. Behavior Documentation

### 1. Behavior Inventory

* **Guideline:** All testable behaviors must be documented in a central inventory (e.g., `docs/testing/behavior-inventory.md`).
* **Structure:** Group behaviors by logical domain. Each behavior gets a unique ID.
* **Format:**
  ```markdown
  | ID | Behavior | Tested? | Test |
  |----|----------|---------|------|
  | P1 | Property description | ✅ | `test_function_name` |
  ```

### 2. Behavior Grouping

* **Guideline:** Group behaviors by functional domain (e.g., Memory, I/O, Synchronization).
* **Policy:** As the kernel grows, new domains should be established to maintain a clean hierarchy.

### 3. Behavior ID Format

* **Guideline:** Each behavior ID uses a unique prefix + number.
* **Extension Policy:**
  - Prefixes must be unique across the kernel.
  - Prefixes should reflect the subsystem or domain they describe.
  - If a single letter is exhausted or ambiguous, use multi-letter prefixes.
* **Example:** `[MMU1]` = MMU behavior #1

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

* **Guideline:** Tests exist at multiple levels (Unit, Integration, System). Leverage Rust's native testing support.
* **Levels:**
  - **Unit Tests:** Isolated logic in `src/` modules using `#[test]`. Use mocks/fakes for hardware dependencies.
  - **Integration Tests:** Verifying interaction between crates in the workspace.
  - **Behavioral Verification:** Kernel boot output comparison against golden references.
  - **Static Verification:** Enforcing rules via custom `clippy` lints or source analysis.
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

* **Guideline:** Regression tests should be centralized in the project's testing harness.
* **Format:** Static analysis of source files or environment state, rather than just runtime execution.
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

### 12. Golden Reference Updates

* **Guideline:** Maintain "Golden" files (expected outputs) for behavioral verification.
* **Update Process:**
  1. Execute the system in a deterministic environment.
  2. Verify output manually or via validated tools.
  3. Update the reference files with the new baseline.
  4. Document the rationale for the change in the commit history.

### 13. Test Failure Response

* **Guideline:** When tests fail:
  1. **Unit test fails:** Bug in implementation. Fix the code.
  2. **Behavior test fails:** Either bug OR golden file needs update. Investigate.
  3. **Regression test fails:** Previously-fixed bug has returned. Root cause analysis required.

### 14. Standard Test Interface

* **Guideline:** The project must provide a unified interface via `cargo` for executing tests at all levels.
* **Standard Commands:**
  ```bash
  cargo test                # Run all isolated unit tests
  cargo xtask test all      # Run full suite (unit + behavior + regression)
  cargo xtask test behavior # System-level behavior verification
  cargo xtask test regress  # Static analysis and regression checks
  ```

## VI. Diagnostic Output & System Silence

### 15. Silence by Default

* **Guideline:** Under normal operation, the system should remain silent. Output is reserved for errors or explicitly requested diagnostics.
* **Implementation Pattern:** Use diagnostic-level macros or logging levels to separate success markers from critical errors.
* **Verification:** Automated tests should verify that "Success" paths do not pollute the primary log buffer.

### 16. Diagnostic Logging

* **Guideline:** Provide a mechanism to enable verbose output for development and verification without modifying code.
* **Rationale:** Ensures that "Golden Reference" tests have enough data to verify behavior while maintaining production silence.

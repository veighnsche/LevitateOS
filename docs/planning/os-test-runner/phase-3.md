# Phase 3: Implementation - OS Test Runner Mode

## Overview

This phase implements the test runner mode for LevitateOS.

## Steps

### Step 1: Create test_runner binary
Create `userspace/levbox/src/bin/test_runner.rs` that orchestrates running all `*_test` binaries.

### Step 2: Create init-test crate  
Create `userspace/init-test/` as a minimal init that spawns test_runner instead of shell.

### Step 3: Add xtask test initramfs builder
Modify `xtask/src/build.rs` to build a test-specific initramfs with init-test.

### Step 4: Add run test command
Modify `xtask/src/run.rs` and `main.rs` to add `cargo xtask run test`.

### Step 5: Create run-test.sh
Create convenience shell script.

## Detailed UoWs

Since each step is reasonably small, they are included inline below.

---

## Step 1 UoW: test_runner binary

**Goal:** Create a test runner binary that runs all *_test binaries sequentially.

**File:** `userspace/levbox/src/bin/test_runner.rs`

**Implementation:**
```rust
#![no_std]
#![no_main]

extern crate ulib;
use libsyscall::{println, spawn, waitpid, exit};

const TESTS: &[&str] = &[
    "mmap_test",
    "pipe_test",
    "signal_test",
    "clone_test",
];

#[no_mangle]
pub fn main() -> i32 {
    println!("[TEST_RUNNER] Starting OS internal tests...");
    println!("[TEST_RUNNER] Test count: {}", TESTS.len());
    
    let mut passed = 0;
    let mut failed = 0;
    
    for test in TESTS {
        println!("");
        println!("[TEST_RUNNER] ----------------------------------------");
        println!("[TEST_RUNNER] Running: {}", test);
        println!("[TEST_RUNNER] ----------------------------------------");
        
        let pid = spawn(test);
        if pid < 0 {
            println!("[TEST_RUNNER] {}: SPAWN_FAILED ({})", test, pid);
            failed += 1;
            continue;
        }
        
        let status = waitpid(pid as usize);
        if status == 0 {
            println!("[TEST_RUNNER] {}: PASS", test);
            passed += 1;
        } else {
            println!("[TEST_RUNNER] {}: FAIL (exit={})", test, status);
            failed += 1;
        }
    }
    
    println!("");
    println!("[TEST_RUNNER] ========================================");
    println!("[TEST_RUNNER] SUMMARY: {}/{} tests passed", passed, passed + failed);
    println!("[TEST_RUNNER] ========================================");
    
    if failed > 0 {
        println!("[TEST_RUNNER] RESULT: FAILED");
        1
    } else {
        println!("[TEST_RUNNER] RESULT: PASSED");
        0
    }
}
```

---

## Step 2 UoW: init-test crate

**Goal:** Create minimal init that spawns test_runner and exits.

**Files:**
- `userspace/init-test/Cargo.toml`
- `userspace/init-test/src/main.rs`
- `userspace/init-test/build.rs`
- `userspace/init-test/link.ld`

**Note:** Copy structure from `userspace/init/` but change behavior.

---

## Step 3 UoW: xtask test initramfs

**Goal:** Add function to build test-specific initramfs.

**File:** `xtask/src/build.rs`

Add:
- `build_test_initramfs()` - Creates initramfs with init-test as `init`
- Uses same binaries but renames init-test → init

---

## Step 4 UoW: run test command

**Goal:** Add `cargo xtask run test` command.

**Files:**
- `xtask/src/run.rs` - Add `Test` variant, `run_qemu_test()` function
- `xtask/src/main.rs` - Wire up command

**Behavior:**
1. Build kernel
2. Build test initramfs
3. Run QEMU headless
4. Capture output
5. Parse for SUMMARY line
6. Report pass/fail

---

## Step 5 UoW: run-test.sh

**Goal:** Create shell script wrapper.

**File:** `run-test.sh`

```bash
#!/bin/bash
# run-test.sh - Run LevitateOS internal tests
exec cargo xtask run test "$@"
```

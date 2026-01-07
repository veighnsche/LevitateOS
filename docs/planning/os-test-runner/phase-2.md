# Phase 2: Design - OS Test Runner Mode

## Proposed Solution

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  cargo xtask run test                                       │
│    │                                                        │
│    ├── Build kernel (normal)                                │
│    ├── Build userspace with test_runner binary              │
│    ├── Create initramfs with init-test (not init)           │
│    └── Run QEMU headless, capture serial output             │
│                                                             │
│  QEMU boots → kernel → spawns init-test → runs tests → exit │
└─────────────────────────────────────────────────────────────┘
```

### Solution Components

1. **`init-test` binary** - Alternative init that runs tests instead of shell
2. **`test_runner` binary** - Orchestrates all test binaries, collects results
3. **`cargo xtask run test`** - New command to build and run test mode
4. **`run-test.sh`** - Shell script wrapper for convenience

## User-Facing Behavior

```bash
# Run all internal OS tests
$ cargo xtask run test

# Output (all to stdout/serial):
[BOOT] Stage 1: Early HAL (SEC)
...
[SUCCESS] LevitateOS System Ready.
--------------------------------------
[TEST_RUNNER] Starting OS internal tests...
[TEST_RUNNER] Running: mmap_test
[mmap_test] Starting memory syscall tests...
[mmap_test] Test 1: PASS
[mmap_test] Test 2: PASS
...
[mmap_test] Results: 5 passed, 0 failed
[TEST_RUNNER] mmap_test: PASS

[TEST_RUNNER] Running: pipe_test
...
[TEST_RUNNER] pipe_test: PASS

[TEST_RUNNER] Running: signal_test
...
[TEST_RUNNER] signal_test: PASS

[TEST_RUNNER] ========================================
[TEST_RUNNER] SUMMARY: 4/4 tests passed
[TEST_RUNNER] ========================================
[TEST_RUNNER] Shutting down...

# QEMU exits, xtask reports success/failure
✅ All OS internal tests passed!
```

## System Behavior

### Boot Sequence (Test Mode)

1. Kernel boots normally (no changes)
2. Kernel spawns `init-test` (instead of `init`)
3. `init-test` spawns `test_runner`
4. `test_runner` sequentially spawns each test binary
5. `test_runner` waits for each test to complete
6. `test_runner` prints summary
7. `test_runner` calls `exit(0)` or `exit(1)`
8. System halts (via PSCI or infinite loop)

### Test Detection

`test_runner` will look for binaries matching pattern `*_test` in initramfs:
- `mmap_test`
- `pipe_test`
- `signal_test`
- `clone_test`

## API Design

### `init-test` (userspace/init-test/src/main.rs)

```rust
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("[INIT-TEST] Starting test mode...");
    
    let runner_pid = spawn("test_runner");
    if runner_pid < 0 {
        println!("[INIT-TEST] ERROR: Failed to spawn test_runner");
        exit(1);
    }
    
    // Wait for test_runner to complete
    let status = waitpid(runner_pid);
    
    // Shutdown system
    println!("[INIT-TEST] Tests complete, shutting down...");
    exit(status);
}
```

### `test_runner` (userspace/levbox/src/bin/test_runner.rs)

```rust
const TESTS: &[&str] = &[
    "mmap_test",
    "pipe_test", 
    "signal_test",
    "clone_test",
];

fn main() -> i32 {
    println!("[TEST_RUNNER] Starting OS internal tests...");
    
    let mut passed = 0;
    let mut failed = 0;
    
    for test in TESTS {
        println!("[TEST_RUNNER] Running: {}", test);
        
        let pid = spawn(test);
        if pid < 0 {
            println!("[TEST_RUNNER] {}: SPAWN_FAILED", test);
            failed += 1;
            continue;
        }
        
        let status = waitpid(pid);
        if status == 0 {
            println!("[TEST_RUNNER] {}: PASS", test);
            passed += 1;
        } else {
            println!("[TEST_RUNNER] {}: FAIL (exit={})", test, status);
            failed += 1;
        }
    }
    
    println!("[TEST_RUNNER] ========================================");
    println!("[TEST_RUNNER] SUMMARY: {}/{} tests passed", passed, passed + failed);
    println!("[TEST_RUNNER] ========================================");
    
    if failed > 0 { 1 } else { 0 }
}
```

### xtask Integration

```rust
// xtask/src/run.rs
pub enum RunCommands {
    Default,
    Pixel6,
    Vnc,
    Term,
    Test,  // NEW
}

// xtask/src/main.rs
run::RunCommands::Test => {
    run::run_qemu_test()?;
}
```

## Behavioral Decisions

### Q1: How to activate test mode?
**Decision:** Separate initramfs with `init-test` as the init binary.
- xtask builds a test-specific initramfs
- No kernel changes needed
- Clean separation

### Q2: Sequential or parallel tests?
**Decision:** Sequential.
- Simpler to implement
- Easier to debug failures
- Tests are fast anyway (<5s each)

### Q3: How to signal completion?
**Decision:** Pattern detection + timeout.
- xtask looks for `[TEST_RUNNER] SUMMARY:` in output
- Also has timeout fallback (30s)
- Parse pass/fail count from summary line

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| Test binary not found | Print SPAWN_FAILED, count as failure |
| Test hangs | Timeout at xtask level (30s total) |
| Test crashes | waitpid returns non-zero, count as failure |
| All tests pass | Exit 0, QEMU terminates |
| Any test fails | Exit 1, QEMU terminates |

## Design Alternatives Considered

### Alternative A: Modify existing init
- **Rejected:** Complicates init, risk of breaking normal boot

### Alternative B: Kernel feature flag
- **Rejected:** Requires separate kernel build, more complex

### Alternative C: Command-line argument to init
- **Rejected:** Would need kernel cmdline parsing, more complex

## Implementation Plan

### Phase 3 Steps

1. **Step 1:** Create `test_runner` binary in levbox
2. **Step 2:** Create `init-test` binary (copy of init, spawns test_runner)
3. **Step 3:** Add `cargo xtask run test` command
4. **Step 4:** Create `run-test.sh` wrapper script
5. **Step 5:** Test and verify

### Files to Create/Modify

| File | Action |
|------|--------|
| `userspace/levbox/src/bin/test_runner.rs` | CREATE |
| `userspace/init-test/` | CREATE (new crate) |
| `xtask/src/run.rs` | MODIFY (add Test variant) |
| `xtask/src/main.rs` | MODIFY (wire up run test) |
| `xtask/src/build.rs` | MODIFY (build test initramfs) |
| `run-test.sh` | CREATE |

## Open Questions (Resolved)

All design questions resolved - ready for implementation.

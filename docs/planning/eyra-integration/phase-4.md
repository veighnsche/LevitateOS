# Phase 4: Integration & Testing — Run Eyra on LevitateOS

**TEAM_351** | Eyra Integration Plan  
**Created:** 2026-01-09  
**Depends on:** Phase 3 (Binary built)

---

## 1. Objective

Run the Eyra test binary on LevitateOS and verify `std` functionality works.

---

## 2. Test Strategy

### Tier 1: Basic Execution
- Binary loads and starts
- `println!` works (write syscall)
- Clean exit (exit_group syscall)

### Tier 2: Runtime Features
- `std::env::args()` works (stack layout, auxv)
- `std::time::Instant::now()` works (clock_gettime)
- Random numbers work (getrandom)

### Tier 3: Threading
- `std::thread::spawn` works (clone, TLS, futex)
- Thread joins successfully
- No deadlocks or crashes

### Tier 4: File I/O
- `std::fs::write` works
- `std::fs::read` works
- `std::fs::metadata` works

---

## 3. Steps

### Step 1: Boot and Run Basic Test

**Commands:**
```bash
# Build everything
cargo xtask build --arch aarch64

# Run with VNC (interactive)
cargo xtask run-vnc --arch aarch64

# In LevitateOS shell:
/eyra-hello
```

**Expected Output:**
```
Hello from Eyra on LevitateOS!
argc = 1
Eyra test complete!
```

### Step 2: Debug Failures

If the binary crashes, check:

1. **Kernel logs** — Look for "Unknown syscall" messages
2. **Exit code** — Non-zero indicates error
3. **Crash location** — Use serial output for clues

**Common issues and fixes:**

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| "Unknown syscall N" | Missing syscall | Implement it |
| Immediate crash | TLS setup failed | Check arch_prctl/TPIDR |
| No output | write syscall broken | Debug sys_write |
| Hang on exit | exit_group missing | Already implemented |

### Step 3: Iterate on Test Program

Once basic hello works, extend the test:

```rust
fn main() {
    // Tier 1: Basic
    println!("Hello from Eyra!");
    
    // Tier 2: Runtime
    let now = std::time::Instant::now();
    println!("Time works: {:?}", now.elapsed());
    
    // Tier 3: Threading
    let handle = std::thread::spawn(|| {
        println!("Thread spawned!");
        42
    });
    let result = handle.join().unwrap();
    println!("Thread returned: {}", result);
    
    // Tier 4: File I/O
    std::fs::write("/tmp/test.txt", "Hello").unwrap();
    let content = std::fs::read_to_string("/tmp/test.txt").unwrap();
    assert_eq!(content, "Hello");
    println!("File I/O works!");
    
    println!("All tests passed!");
}
```

### Step 4: Create Automated Test

Add to `cargo xtask test`:

```rust
// xtask/src/tests/eyra.rs
pub fn test_eyra_hello() -> Result<()> {
    // Boot LevitateOS
    // Run /eyra-hello
    // Check output contains "Eyra test complete!"
    // Check exit code is 0
}
```

---

## 4. UoWs (Units of Work)

### UoW 4.1: First boot test

**Tasks:**
1. Build kernel with eyra-hello in initramfs
2. Boot with VNC
3. Run `/eyra-hello` from shell
4. Document output (success or failure)

**Exit Criteria:**
- Output documented
- If failed, failure mode identified

### UoW 4.2: Debug and fix failures

**Tasks:**
1. Analyze failure (kernel logs, exit code)
2. Identify missing/broken syscall
3. Implement fix
4. Retry test

**Exit Criteria:**
- Basic hello world prints and exits cleanly

### UoW 4.3: Extend test coverage

**Tasks:**
1. Add time test
2. Add threading test
3. Add file I/O test
4. Document which features work

**Exit Criteria:**
- Feature matrix documented
- Known limitations listed

### UoW 4.4: Create automated test

**Tasks:**
1. Add eyra test to xtask
2. Run in CI if possible
3. Add to regression suite

**Exit Criteria:**
- `cargo xtask test eyra` runs and passes

---

## 5. Success Criteria

- [ ] Basic hello world runs
- [ ] `println!` outputs to console
- [ ] Clean exit (no crash)
- [ ] `std::env::args()` returns correct argc
- [ ] At least Tier 1-2 features work
- [ ] Known limitations documented

---

## 6. Known Limitations (Expected)

These may not work initially and are acceptable to defer:

| Feature | Status | Notes |
|---------|--------|-------|
| `std::process::Command` | ❌ | Needs fork/execve |
| `std::net` | ❌ | Needs network stack |
| `std::fs` (real disk) | ❌ | Only tmpfs supported |
| Dynamic linking | ❌ | Eyra is static-only |

---

## 7. Next Phase

**Phase 5:** Document the integration, update ROADMAP, handoff.

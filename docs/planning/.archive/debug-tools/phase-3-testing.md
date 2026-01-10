# Phase 3 — Integration Testing for Debug Tools

**Parent:** `docs/planning/debug-tools/`  
**Author:** TEAM_325  
**Status:** PLANNING

---

## Feature Summary

**Problem:** The `debug regs` and `debug mem` commands have no automated tests. They were manually verified but could regress silently.

**Solution:** Add integration tests that:
1. Start a QEMU session with QMP enabled
2. Execute debug commands against the live VM
3. Verify output format and content
4. Clean up the session

**Who benefits:**
- CI/CD pipeline — catches regressions automatically
- Developers — confidence that debug tools work
- AI agents — can verify debug tool functionality

---

## Success Criteria

| Criterion | Verification |
|-----------|--------------|
| `debug regs` returns register data | Test parses output for expected register names (RAX, RBX, etc. or X0, X1, etc.) |
| `debug mem` returns hex dump | Test verifies hex dump format with addresses |
| Tests run in CI | `cargo xtask test debug` passes |
| Tests work on both architectures | Tested with `--arch aarch64` and `--arch x86_64` |
| Tests are deterministic | No flaky failures |

---

## Current State Analysis

### Existing test patterns:

| Test | Pattern | What it does |
|------|---------|--------------|
| `serial_input.rs` | Start QEMU, pipe commands, check output | Verifies serial I/O |
| `behavior.rs` | Start QEMU, capture output, compare to golden file | Regression protection |
| `keyboard_input.rs` | Start QEMU with VNC, send keys via QMP | Tests QMP sendkey |

### Key insight:
`keyboard_input.rs` already uses QMP for sending keys. We can reuse this pattern for debug commands.

### Gaps:
- No test for `debug regs`
- No test for `debug mem`
- No `cargo xtask test debug` command

---

## Codebase Reconnaissance

### Files to create:
| File | Purpose |
|------|---------|
| `xtask/src/tests/debug_tools.rs` | Integration tests for debug commands |

### Files to modify:
| File | Change |
|------|--------|
| `xtask/src/tests/mod.rs` | Add `pub mod debug_tools;` |
| `xtask/src/main.rs` | Add `"debug"` to test suite options |

### Dependencies:
- `QmpClient` from `support/qmp.rs`
- `QemuBuilder` from `qemu/builder.rs`
- Shell session pattern from `shell/session.rs`

---

## Constraints

1. **Timeout handling** — VM boot takes 5-10s, tests must wait appropriately
2. **Cleanup** — Tests must kill QEMU even on failure
3. **Port conflicts** — Use unique QMP socket paths per test
4. **Determinism** — Memory contents vary, but format should be consistent
5. **CI compatibility** — Must work headless without display

---

## Test Design

### Test 1: `test_debug_regs`

```
1. Start QEMU with QMP socket
2. Wait for boot
3. Connect to QMP
4. Execute `human-monitor-command` with `info registers`
5. Verify output contains expected register names
6. Kill QEMU
```

**Expected registers:**
- aarch64: `X0`, `X1`, `PC`, `SP`, `PSTATE`
- x86_64: `RAX`, `RBX`, `RCX`, `RIP`, `RSP`, `RFLAGS`

### Test 2: `test_debug_mem`

```
1. Start QEMU with QMP socket
2. Wait for boot
3. Connect to QMP
4. Execute `memsave` for known address (e.g., 0x0 for interrupt vectors)
5. Verify output is hex dump format
6. Kill QEMU
```

**Verification:**
- Output contains address prefix (e.g., `0x00000000:`)
- Output contains hex bytes (e.g., `48 8b 05`)
- Output contains ASCII column (e.g., `|H......|`)

---

## Design Decisions (User Answered)

| Question | Decision |
|----------|----------|
| Q1: Part of `test all`? | **No** — Separate `cargo xtask test debug` command |
| Q2: Test both archs? | **Yes** — Test both aarch64 and x86_64 |
| Q3: Golden files? | **Yes** — Golden file comparison with flexibility for varying values |

### Testing Approach

Per user guidance: Use deterministic test scenarios like QEMU's own testing approach.

**Strategy:**
1. Capture register state at a known point (immediately after QMP connection, before randomness)
2. Use golden files that match on **structure** (register names, format) with wildcards for values
3. For memory: dump a known fixed address (e.g., 0x0 interrupt vectors or kernel entry point)

---

## Implementation Steps

### Step 1: Create test module
- Create `xtask/src/tests/debug_tools.rs`
- Add test harness (start VM, run test, cleanup)

### Step 2: Implement `test_debug_regs`
- Start QEMU with QMP
- Call `debug::regs()` or directly use QMP
- Verify register names in output

### Step 3: Implement `test_debug_mem`
- Start QEMU with QMP
- Call `debug::mem()` for address 0x0
- Verify hex dump format

### Step 4: Wire up to xtask
- Add `debug` to test suite enum
- Add to `cargo xtask test --help`

### Step 5: Verify on both architectures
- Run with `--arch aarch64`
- Run with `--arch x86_64`

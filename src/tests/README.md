# tests/

Automated test suites for LevitateOS.

## Files

| File | Description |
|------|-------------|
| `mod.rs` | Module exports |
| `backspace.rs` | **NEW** Backspace regression test (prevents ^H echo bug) |
| `common.rs` | **NEW** Shared test utilities (QemuSession, etc.) |
| `screenshot.rs` | **NEW** Unified screenshot tests (userspace, levitate, alpine) |
| `unit.rs` | Unit tests (cargo test) |
| `behavior.rs` | Behavior tests with golden file comparison |
| `regression.rs` | Regression test suite |
| `debug_tools.rs` | Debug tools integration tests |
| `serial_input.rs` | Serial console input tests |
| `keyboard_input.rs` | Keyboard input tests |
| `shutdown.rs` | Graceful shutdown tests |

## Running Tests

| Command | Description |
|---------|-------------|
| `cargo xtask test` | Run complete test suite |
| `cargo xtask test unit` | Unit tests only |
| `cargo xtask test behavior` | Behavior tests with golden files |
| `cargo xtask test regress` | Regression tests |
| `cargo xtask test debug` | Debug tools integration tests |
| `cargo xtask test serial` | Serial input tests |
| `cargo xtask test keyboard` | Keyboard input tests |
| `cargo xtask test shutdown` | Shutdown tests |
| **`cargo xtask test backspace`** | **Backspace regression test** |
| **`cargo xtask test screenshot`** | **All screenshot tests** |
| **`cargo xtask test userspace`** | **Userspace tests + screenshot** |
| `cargo xtask test levitate` | LevitateOS display tests |
| `cargo xtask test alpine` | Alpine Linux reference tests |

## Golden Files

Located in `tests/`:
- `golden_boot.txt` — aarch64 boot log
- `golden_boot_x86_64.txt` — x86_64 boot log
- `golden_debug_regs_*.txt` — Register dump format
- `golden_debug_mem_*.txt` — Memory dump format

### Updating Golden Files

```bash
cargo xtask test behavior --update
cargo xtask test debug --update
```

## Debug Tools Tests

Uses **bare QEMU** (no guest OS) for deterministic testing:
- Starts QEMU paused (`-S`)
- Queries initial CPU/memory state via QMP
- Compares against golden files

This ensures tests don't break when LevitateOS changes.

## Alpine Screenshot Tests

Uses **Alpine Linux** (stable external image) for screenshot testing:

```bash
# Download Alpine images first
./tests/images/download.sh

# Run screenshot tests
cargo xtask test screenshot
```

Captures:
- `alpine_{arch}_shell.ppm` — Shell with date/time
- `alpine_{arch}_ls.ppm` — ls output

## LevitateOS Display Tests

Tests LevitateOS display output on both architectures:

```bash
cargo xtask test levitate
```

Captures:
- `levitate_aarch64.png` — aarch64 display (should show shell)
- `levitate_x86_64.png` — x86_64 display (known issue: black screen)

## History

- TEAM_030: Initial behavior tests
- TEAM_139: Serial input tests
- TEAM_156: Keyboard input tests
- TEAM_325: Debug tools integration tests, Alpine screenshot tests
- TEAM_326: Added LevitateOS display tests, updated command references

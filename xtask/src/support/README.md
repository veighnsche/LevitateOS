# support/

Shared utilities for xtask commands.

## Files

| File | Description |
|------|-------------|
| `mod.rs` | Module exports |
| `clean.rs` | Cleanup utilities (kill QEMU, remove artifacts) |
| `preflight.rs` | Pre-flight checks (tools, targets, components) |
| `qmp.rs` | QMP client for QEMU communication |

## clean.rs

| Function | Description |
|----------|-------------|
| `clean(arch)` | Remove build artifacts and QEMU locks |
| `kill_qemu(arch)` | Kill any running QEMU instances |

## preflight.rs

Checks before running commands:
- Required tools exist (`cargo`, `qemu-system-*`, etc.)
- Rust targets installed (`aarch64-unknown-none`, `x86_64-unknown-none`)
- Rust components installed (`rust-src`, `llvm-tools`)

## qmp.rs

QMP (QEMU Machine Protocol) client:

```rust
let mut client = QmpClient::connect("./qmp.sock")?;
let result = client.execute("screendump", Some(json!({"filename": "out.ppm"})))?;
```

Used by:
- `vm regs` / `vm mem` — Register and memory inspection
- `vm send` / `vm screenshot` — Session interaction
- Screenshot tests

## Commands

| Command | Description |
|---------|-------------|
| `cargo xtask check` | Run preflight checks |
| `cargo xtask clean` | Clean artifacts and locks |
| `cargo xtask kill` | Kill running QEMU instances |

## History

- TEAM_322: Organized into submodule
- TEAM_326: Renamed preflight → check, updated for vm module

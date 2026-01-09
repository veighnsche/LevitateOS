# qemu/

QEMU management with builder pattern for command line construction.

## Files

| File | Description |
|------|-------------|
| `mod.rs` | Module exports |
| `builder.rs` | `QemuBuilder` — fluent API for QEMU args |
| `profile.rs` | `QemuProfile` — preset configurations |

## QemuBuilder

Fluent builder for constructing QEMU command lines:

```rust
let mut cmd = QemuBuilder::new(Arch::Aarch64, QemuProfile::Default)
    .display_vnc()
    .enable_qmp("./qmp.sock")
    .build()?;
```

## QemuProfile

| Profile | Description |
|---------|-------------|
| `Default` | Standard aarch64 virt machine |
| `Pixel6` | 8GB RAM, 8 cores (Pixel 6 simulation) |
| `GicV3` | GICv3 interrupt controller testing |
| `X86_64` | x86_64 q35 machine |

## Arch

| Arch | QEMU Binary |
|------|-------------|
| `Aarch64` | `qemu-system-aarch64` |
| `X86_64` | `qemu-system-x86_64` |

## History

- TEAM_322: Extracted from run.rs to eliminate duplication

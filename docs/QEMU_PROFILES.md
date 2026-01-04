# QEMU Hardware Profiles

**Created By**: TEAM_042 (2026-01-04)
**Updated**: 2026-01-04 (Verified QEMU 10.1+ capabilities, added mitigations)

LevitateOS supports multiple QEMU profiles to test on different hardware configurations.

---

## Available Profiles

### Default Profile
- **CPU**: cortex-a53 (1 core)
- **RAM**: 512MB
- **GIC**: v2 (default)
- **Use Case**: Quick iteration, CI/CD, low memory testing

```bash
cargo xtask run           # Uses default profile
./run.sh                  # Uses default profile
```

### Pixel 6 Profile
- **CPU**: cortex-a76 × 8 cores (2 clusters × 4 cores)
- **RAM**: 8GB
- **GIC**: v3
- **Use Case**: Target hardware simulation, memory allocator testing

```bash
cargo xtask run-pixel6    # Uses Pixel 6 profile
./run-pixel6.sh           # Uses Pixel 6 profile
```

---

## Hardware Mapping (Updated)

| Pixel 6 (Tensor GS101) | QEMU Equivalent | Status |
|------------------------|-----------------|--------|
| 2× Cortex-X1 @ 2.8GHz | cortex-a76 (very close) | ✅ |
| 2× Cortex-A76 @ 2.25GHz | cortex-a76 (**exact match**) | ✅ |
| 4× Cortex-A55 @ 1.8GHz | cortex-a76 (faster, but same ISA) | ⚠️ |
| 8 cores | `-smp 8` | ✅ |
| 8GB LPDDR5 | `-m 8G` | ✅ |
| Mali-G78 MP20 | `virtio-gpu-device` | ⚠️ |
| UFS 3.1 | `virtio-blk-device` | ⚠️ |
| GICv3 | GICv2 (kernel driver TODO) | ⚠️ |

---

## Mitigations Applied

### 1. CPU Type (Previously: "No cortex-a76")
**Status**: ✅ **RESOLVED**

QEMU 10.1+ includes `cortex-a76` which is an **exact match** for Pixel 6's medium cores and very close to X1.

```bash
qemu-system-aarch64 -cpu cortex-a76  # Now used!
```

### 2. big.LITTLE Heterogeneous Cores
**Status**: ⚠️ **MITIGATED** (not fully resolved)

QEMU cannot mix CPU types, but we use **cluster topology** to simulate scheduling domains:

```bash
-smp sockets=1,clusters=2,cores=4,threads=1
```

This gives the guest kernel cluster-aware scheduling hints via ACPI PPTT and DT cpu-map. While all cores run at the same speed, the scheduler will treat them as two distinct groups.

### 3. GICv3 Support
**Status**: ⚠️ **KERNEL DRIVER NEEDED**

QEMU supports GICv3 (up to 512 CPUs), but the LevitateOS GIC driver only implements GICv2.
- GICv2 uses memory-mapped CPU interface (`GICC_BASE`)
- GICv3 uses system registers (`ICC_*`)
- **Workaround**: Using GICv2 which supports up to 8 CPUs (matches Pixel 6 core count)
- **TODO**: Implement GICv3 driver for full Pixel 6 compatibility

### 4. Mali GPU
**Status**: ⚠️ **NOT MITIGATABLE**

No Mali GPU emulation exists. VirtIO-GPU provides a functional framebuffer but not Mali-specific features (compute shaders, etc.). GPU-specific code must be tested on real hardware.

---

## Memory Layout Comparison

### QEMU `virt` Machine
```
0x00000000 - 0x3FFFFFFF  : Flash, GIC, devices (1GB)
0x40000000 - 0xFFFFFFFF  : RAM (up to 3GB in low region)
0x100000000+             : RAM (highmem for >4GB)
```

### Pixel 6 (Approximate)
```
0x00000000 - 0x7FFFFFFF  : SoC peripherals, secure memory (2GB)
0x80000000 - 0x27FFFFFFF : RAM (8GB starting at 2GB offset)
```

**Note**: Both support >4GB RAM with highmem regions — critical for buddy allocator testing.

---

## Remaining Limitations

| Limitation | Impact | Workaround |
|------------|--------|------------|
| No true big.LITTLE | Scheduler can't differentiate core performance | All cores same type |
| GICv3 not implemented | Using GICv2 (8 CPU limit) | Matches Pixel 6 count |
| No Mali GPU | Can't test GPU compute | Test on real hardware |
| No Tensor TPU | Can't test ML accelerator | N/A for kernel |
| Timing not cycle-accurate | Performance numbers invalid | Use for correctness only |

---

## When to Use Each Profile

| Scenario | Profile |
|----------|---------|
| Quick build verification | Default |
| CI/CD pipelines | Default |
| Memory allocator testing | **Pixel 6** |
| 8-core SMP testing | **Pixel 6** |
| GICv3 interrupt testing | **Pixel 6** |
| Cluster-aware scheduler testing | **Pixel 6** |
| Low memory edge cases | Default |
| Pre-hardware bringup testing | **Pixel 6** |

---

## Configuration Files

- `qemu/pixel6.conf` - Pixel 6 profile configuration
- `run-pixel6.sh` - Shell script for Pixel 6 profile
- `xtask/src/main.rs` - `QemuProfile` enum with all profiles

---

## QEMU Version Requirements

Minimum: **QEMU 7.2+** (cluster topology support)
Recommended: **QEMU 10.0+** (latest CPU models, bug fixes)

Check your version:
```bash
qemu-system-aarch64 --version
```

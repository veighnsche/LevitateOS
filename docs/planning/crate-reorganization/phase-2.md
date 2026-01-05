# Phase 2: Structural Extraction

**Parent:** [README.md](./README.md)  
**Depends:** Phase 1 complete  
**Status:** Planned

---

## Target Design

### New Crate Layout

```
levitate-utils/          # Foundation (no changes)
levitate-hal/            # CPU/Platform HAL only
levitate-virtio/         # VirtIO transport + HAL impl
levitate-drivers-gpu/    # VirtIO GPU driver (renamed from levitate-virtio-gpu)
levitate-drivers-blk/    # VirtIO Block driver (extracted from kernel)
levitate-drivers-net/    # VirtIO Net driver (extracted from kernel)
levitate-drivers-input/  # VirtIO Input driver (extracted from kernel)
levitate-terminal/       # Terminal subsystem (no changes)
levitate-fs/             # Filesystem subsystem (new)
```

### Responsibility Matrix

| Crate | Owns | Depends On |
|-------|------|------------|
| `levitate-utils` | Spinlock, RingBuffer, CPIO, hex | None |
| `levitate-hal` | MMU, GIC, Timer, UART, Console, FDT, Interrupts | levitate-utils |
| `levitate-virtio` | VirtQueue, MmioTransport, VirtioHal trait impl | levitate-hal |
| `levitate-drivers-gpu` | VirtIO GPU protocol, framebuffer | levitate-virtio, embedded-graphics |
| `levitate-drivers-blk` | VirtIO Block protocol, read/write | levitate-virtio |
| `levitate-drivers-net` | VirtIO Net protocol, MAC, send/recv | levitate-virtio |
| `levitate-drivers-input` | VirtIO Input protocol, events | levitate-virtio |
| `levitate-terminal` | Terminal emulator, ANSI parsing | levitate-utils, embedded-graphics |
| `levitate-fs` | FAT32, ext4 abstractions | levitate-drivers-blk |

---

## Extraction Strategy

### Order of Operations

1. **Fix VirtQueue in levitate-virtio** (prerequisite)
2. **Move VirtIO HAL** from levitate-hal to levitate-virtio
3. **Fix and rename levitate-virtio-gpu** → levitate-drivers-gpu
4. **Extract levitate-drivers-blk** from kernel/src/block.rs
5. **Extract levitate-drivers-net** from kernel/src/net.rs
6. **Extract levitate-drivers-input** from kernel/src/input.rs
7. **Create levitate-fs** to wrap filesystem code

### Coexistence Strategy

During extraction, old and new paths coexist:
- Old: `levitate-gpu` (working)
- New: `levitate-drivers-gpu` (being fixed)

Switch happens atomically in Phase 3 after new driver is verified.

---

## Steps

### Step 1: Fix VirtQueue DMA Bugs
**File:** `phase-2-step-1.md`

Root cause analysis from TEAM_100:
- VirtQueue stores descriptors in local memory
- `addresses()` returns virtual addresses
- Device needs physical addresses for DMA
- Volatile reads/writes needed for device-written memory

Tasks:
1. Allocate queue structures in DMA-safe memory
2. Use HAL's virt_to_phys for all descriptor addresses
3. Volatile read/write for used ring
4. Test with QEMU

---

### Step 2: Move VirtIO HAL Implementation
**File:** `phase-2-step-2.md`

Move `levitate-hal/src/virtio.rs` → `levitate-virtio/src/hal_impl.rs`

Tasks:
1. Create `hal_impl.rs` in levitate-virtio
2. Move `LevitateVirtioHal` implementation
3. Move `StaticMmioTransport` type alias
4. Update levitate-hal to remove virtio.rs
5. Update imports in all dependent crates

---

### Step 3: Rename and Finalize GPU Driver
**File:** `phase-2-step-3.md`

Rename `levitate-virtio-gpu` → `levitate-drivers-gpu`

Tasks:
1. Rename directory
2. Update Cargo.toml (name field)
3. Update workspace Cargo.toml
4. Update all imports
5. Verify builds

---

### Step 4: Extract Block Driver
**File:** `phase-2-step-4.md`

Create `levitate-drivers-blk` from `kernel/src/block.rs`

Tasks:
1. Create new crate with Cargo.toml
2. Move block driver code
3. Define public API
4. Update kernel to use new crate
5. Remove kernel's direct virtio-drivers dep for block

---

### Step 5: Extract Net Driver (OPTIONAL - defer if needed)
**File:** `phase-2-step-5.md`

Create `levitate-drivers-net` from `kernel/src/net.rs`

> **Note:** This step can be deferred. Net driver is minimally used and not blocking core goals.

Tasks:
1. Create new crate with Cargo.toml
2. Move net driver code
3. Define public API
4. Update kernel to use new crate

---

### Step 6: Extract Input Driver (OPTIONAL - defer if needed)
**File:** `phase-2-step-6.md`

Create `levitate-drivers-input` from `kernel/src/input.rs`

> **Note:** This step can be deferred. Input driver works fine embedded in kernel.

Tasks:
1. Create new crate with Cargo.toml
2. Move input driver code
3. Define public API
4. Update kernel to use new crate

---

### Step 7: Create Filesystem Crate (OPTIONAL - defer if needed)
**File:** `phase-2-step-7.md`

Create `levitate-fs` to wrap filesystem abstractions

> **Note:** This step can be deferred. Filesystem abstraction is nice-to-have, not blocking GPU fix.

Tasks:
1. Create new crate with Cargo.toml
2. Define filesystem trait
3. Wrap embedded-sdmmc (FAT32)
4. Wrap ext4-view
5. Update kernel to use levitate-fs
6. Remove kernel's direct embedded-sdmmc/ext4-view deps

---

## Perfection Criteria

- [ ] All new crates build independently
- [ ] Old and new paths coexist without conflict
- [ ] No circular dependencies
- [ ] Each crate has clear, minimal public API
- [ ] Tests pass at each step

# Phase 3: Migration

**TEAM_332** | VirtIO Driver Reorganization  
**TEAM_333** | Reviewed: 2026-01-09

## Migration Strategy

### Order of Migration

Migrate drivers in order of increasing complexity:

1. **virtio-input** - Already has dual transport, good test
2. **virtio-blk** - Simple, few call sites
3. **virtio-net** - Similar to block
4. **virtio-gpu** - Most call sites, most complex

### Migration Pattern

For each driver:

1. Update kernel imports to use new crate
2. Replace old driver code with thin wrapper
3. Verify all tests pass
4. Remove old code in Phase 4

### Breaking Changes Approach

Per Rule 5 (Breaking Changes > Fragile Compatibility):

- **Do not** create compatibility shims
- **Do** update all call sites directly
- **Do** let compiler errors guide migration
- **Do** fix import sites one by one

## Call Site Inventory

### Input Driver Call Sites

| File | Usage | Priority |
|------|-------|----------|
| `kernel/src/syscall/sys.rs` | `input::read_char()` | High |
| `kernel/src/init.rs` | `input::poll()` via timer | High |
| `kernel/src/virtio.rs` | `input::init()` / `input::init_pci()` | High |
| `userspace/init/src/shell.rs` | Indirect via syscall | Low |

### Block Driver Call Sites

| File | Usage | Priority |
|------|-------|----------|
| `kernel/src/fs/fat.rs` | `block::read_block()` | High |
| `kernel/src/fs/vfs/` | Block device trait | High |
| `kernel/src/virtio.rs` | `block::init()` | Medium |

### Network Driver Call Sites

| File | Usage | Priority |
|------|-------|----------|
| `kernel/src/net.rs` | Self-contained | Low |
| `kernel/src/virtio.rs` | `net::init()` | Low |

### GPU Driver Call Sites

| File | Usage | Priority |
|------|-------|----------|
| `kernel/src/terminal.rs` | `gpu::GPU`, `as_display()` | High |
| `kernel/src/init.rs` | `gpu::init()`, `debug_display_status()` | High |
| `kernel/src/input.rs` | `gpu::GPU.lock().dimensions()` | Medium |
| `kernel/src/syscall/sys.rs` | GPU flush on shutdown | Medium |

## Rollback Plan

If migration causes issues:

1. **Git revert** - Each step should be a clean commit
2. **Feature flag** - Can add `#[cfg(feature = "new_drivers")]` temporarily
3. **Dual paths** - Old code kept until Phase 4, can switch back

---

## Phase 3 Steps

### Step 1: Migrate Input Driver

**File:** `phase-3-step-1.md`

Tasks:
1. Update `kernel/src/virtio.rs` to use `virtio_input` crate
2. Update `kernel/src/syscall/sys.rs` imports
3. Update timer interrupt handler
4. Create thin `kernel/src/drivers/input.rs` wrapper
5. Verify keyboard input works on both arches

**Exit Criteria:**
- All input call sites use new crate
- Keyboard works on x86_64 and aarch64
- Tests pass

### Step 2: Migrate Block Driver

**File:** `phase-3-step-2.md`

Tasks:
1. Update `kernel/src/virtio.rs` to use `virtio_blk` crate
2. Update FS layer imports
3. Create thin `kernel/src/drivers/block.rs` wrapper
4. Verify block operations work

**Exit Criteria:**
- All block call sites use new crate
- FS mount works (or fails gracefully as before)
- Tests pass

### Step 3: Migrate Network Driver

**File:** `phase-3-step-3.md`

Tasks:
1. Update `kernel/src/virtio.rs` to use `virtio_net` crate
2. Create thin `kernel/src/drivers/net.rs` wrapper

**Exit Criteria:**
- Network driver uses new crate
- Tests pass

### Step 4: Migrate GPU Driver

**File:** `phase-3-step-4.md`

Tasks:
1. Move `UnifiedDisplay`, `FramebufferGpu` to `virtio-gpu` crate
2. Update `kernel/src/terminal.rs` imports
3. Update `kernel/src/init.rs` GPU calls
4. Create thin `kernel/src/drivers/gpu.rs` wrapper
5. Verify display works on both arches

**Exit Criteria:**
- All GPU call sites use new crate
- Display works on x86_64 (Limine FB) and aarch64 (VirtIO)
- Screenshot tests pass

### Step 5: Update Device Discovery

**File:** `phase-3-step-5.md`

Tasks:
1. Refactor `kernel/src/virtio.rs` to use `virtio-transport`
2. Create unified device enumeration
3. Remove arch-specific discovery code from drivers

**Exit Criteria:**
- Single discovery path for both MMIO and PCI
- All drivers initialized through unified API
- Tests pass

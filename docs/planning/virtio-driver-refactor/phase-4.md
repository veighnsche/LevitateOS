# Phase 4: Cleanup

**TEAM_332** | VirtIO Driver Reorganization  
**TEAM_333** | Reviewed: 2026-01-09

## Dead Code Removal

Per Rule 6 (No Dead Code):

### Files to Delete

| File | Reason |
|------|--------|
| `kernel/src/block.rs` | Replaced by `virtio-blk` crate |
| `kernel/src/input.rs` | Replaced by `virtio-input` crate |
| `kernel/src/net.rs` | Replaced by `virtio-net` crate |
| `kernel/src/gpu.rs` | Replaced by `virtio-gpu` crate |
| `crates/virtio/` | Unused reference code |
| `crates/hal/src/virtio.rs` | HAL parts moved to transport crate |

### Code to Remove from Remaining Files

| File | Remove |
|------|--------|
| `kernel/src/virtio.rs` | Old MMIO scanning, keep unified discovery |
| `kernel/src/main.rs` | Old module declarations |
| `Cargo.toml` | Old crate references |

## Temporary Adapter Removal

During Phase 2-3, we may introduce:

- Re-export shims in old locations → Remove
- `#[deprecated]` markers → Remove deprecated items
- Dual import paths → Consolidate to new paths

## Encapsulation Tightening

### Make Private

| Crate | Item | Reason |
|-------|------|--------|
| `virtio-input` | `InputDevice.inner` | Implementation detail |
| `virtio-blk` | `BlockDevice.inner` | Implementation detail |
| `virtio-transport` | Internal queue management | Not public API |

### Remove Exports

| Crate | Item | Reason |
|-------|------|--------|
| `virtio-transport` | `virtio_drivers` re-exports | Use crate directly if needed |
| `virtio-input` | Internal key constants | Only expose `read_char()` |

## File Size Check

Per Rule 7 (Modular Refactoring):

Target sizes after refactor:

| File | Target Lines |
|------|--------------|
| `virtio-transport/src/lib.rs` | <200 |
| `virtio-input/src/lib.rs` | <150 |
| `virtio-input/src/keymap.rs` | <100 |
| `virtio-blk/src/lib.rs` | <150 |
| `virtio-net/src/lib.rs` | <150 |
| `virtio-gpu/src/lib.rs` | <200 |
| `kernel/src/drivers/*.rs` | <50 each |

---

## Phase 4 Steps

### Step 1: Remove Old Driver Files

**File:** `phase-4-step-1.md`

Tasks:
1. Delete `kernel/src/block.rs`
2. Delete `kernel/src/input.rs`
3. Delete `kernel/src/net.rs`
4. Delete `kernel/src/gpu.rs`
5. Update `kernel/src/main.rs` module declarations
6. Verify build succeeds

**Exit Criteria:**
- Old files deleted
- Build succeeds
- Tests pass

### Step 2: Remove Unused Crates

**File:** `phase-4-step-2.md`

Tasks:
1. Delete `crates/virtio/` directory
2. Remove from workspace Cargo.toml
3. Clean up any references

**Exit Criteria:**
- Dead crate removed
- Workspace builds
- No dangling references

### Step 3: Tighten Visibility

**File:** `phase-4-step-3.md`

Tasks:
1. Review all `pub` items in new crates
2. Make internal items `pub(crate)` or private
3. Remove unnecessary re-exports
4. Add `#![deny(missing_docs)]` to new crates

**Exit Criteria:**
- Minimal public API surface
- All public items documented
- No unnecessary exports

### Step 4: Verify File Sizes

**File:** `phase-4-step-4.md`

Tasks:
1. Check line counts of all new files
2. Split any files >500 lines
3. Ensure logical separation of concerns

**Exit Criteria:**
- All files <500 lines (ideally <300)
- Clear module boundaries

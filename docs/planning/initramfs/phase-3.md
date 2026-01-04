# Initramfs - Phase 3: Implementation

## 1. Step 1: DTB Address Preservation
### Objective
Preserve the DTB address passed by QEMU in `x0` and make it available to the Rust kernel.

### Tasks
- [x] Modify `kernel/src/main.rs` assembly:
  - Add `BOOT_DTB_ADDR` static.
  - In `_start`, save `x0` to `BOOT_DTB_ADDR`.
- [x] Implement `get_dtb_phys` and `get_dtb_virt` in `kernel/src/main.rs`.

## 2. Step 2: DTB Support in HAL
### Objective
Add Devicetree parsing support to `levitate-hal`.

### Tasks
- [x] Add `fdt` crate to `levitate-hal/Cargo.toml`.
- [x] Create `levitate-hal/src/fdt.rs` with helper functions:
  - `get_initrd_range()`: Returns `Option<(usize, usize)>` from the DTB.

## 3. Step 3: CPIO Parser
### Objective
Implement a minimal CPIO New ASCII (`newc`) parser.

### Tasks
- [x] Create `kernel/src/fs/initramfs.rs`.
- [x] Define `CpioHeader` struct matching the format.
- [x] Implement iterator over CPIO entries.
- [x] Implement file lookup by path.

## 4. Step 4: Kernel Integration
### Objective
Glue everything together in `kmain`.

### Tasks
- [x] Call `hal::fdt` parsing logic in `kmain`.
- [x] Initialize `Initramfs` instance.
- [x] Verify by printing the version/content of a test file (`hello.txt`).

## 5. Verification Plan
- [x] Build kernel and test initramfs.
- [x] Run QEMU with `-initrd initramfs.cpio`.
- [x] Observe kernel output for filesystem listing.

# Initramfs - Phase 1: Discovery

## 1. Feature Summary
- **Description**: Load an initial ramdisk (initramfs) from memory, parse its contents, and provide access to the files it contains.
- **Problem Statement**: LevitateOS needs a way to load early userspace processes and configuration without necessarily having a full disk driver or complex filesystem ready. Initramfs provides a simple, memory-resident filesystem for this purpose.
- **Target Audience**: Future userspace implementation (Phase 8) and early boot customization.

## 2. Success Criteria
- [ ] **DTB Retrieval**: The kernel successfully preserves and accesses the Devicetree Blob (DTB) address passed by the bootloader/QEMU.
- [ ] **Initrd Location**: The kernel identifies the start and end addresses of the initramfs from the DTB (`linux,initrd-start`, `linux,initrd-end`).
- [ ] **CPIO Parsing**: The kernel can parse the CPIO (New ASCII Format) archive structure.
- [ ] **File Access**: The kernel can list and read files from the initramfs.
- [ ] **Integration**: The initramfs is accessible via a simple internal API (and eventually a VFS).

## 3. Current State Analysis
- **Boot Protocol**: QEMU's `virt` machine machine passes the DTB address in `x0`.
- **Boot Assembly**: Current `_start` in `kernel/src/main.rs` overwrites `x0` immediately with `mpidr_el1`, losing the DTB pointer.
- **MMU**: The kernel uses a higher-half mapping. Any pointer passed at boot (physical) must be converted to a virtual address after the MMU is enabled.
- **Dependencies**: No current support for DTB or CPIO parsing in the workspace.
- **Filesystem**: FAT32 is implemented but requires VirtIO Block hardware. Initramfs will provide a "hardware-free" early filesystem.

## 4. Codebase Reconnaissance
- **`kernel/src/main.rs`**: Needs modification in `_start` (assembly) to save `x0`.
- **`levitate-hal`**: Likely candidate for DTB parsing logic if it's considered board/arch specific, or a new `levitate-fdt` crate if we want to be modular.
- **`kernel/src/fs/mod.rs`**: Integration point for the new filesystem.

## 5. Constraints
- **`no_std`**: All parsing must be done without the standard library.
- **Memory Safety**: Parsing untrusted blobs (initrd) must be done safely.
- **Efficiency**: Avoid copying file data; provide views into the memory-mapped initrd.

## 6. Steps
### Step 1 – Capture Feature Intent
- [x] Documented in this file.

### Step 2 – Analyze Current State
- [x] Identified `x0` preservation issue.
- [x] Verified QEMU initrd loading behavior:
  - Confirmed `linux,initrd-start` and `linux,initrd-end` are present in DTB when `-initrd` is used.
  - Confirmed raw binary is needed for `dumpdtb` to work reliably in test environment.
- [x] Analyze RAM mapping:
  - Current assembly maps 1GB of RAM starting at `0x40000000`.
  - DTB and Initrd are typically placed within this first GB, so they are already mapped.
  - Need to ensure we don't overwrite them with heap or other data.

### Step 3 – Source Code Reconnaissance
- [x] x0 preservation:
  - Plan: Store `x0` in a `static mut` at the very beginning of `_start`.
- [x] Search for existing `no_std` CPIO and FDT crates:
  - [x] `fdt` crate identified for DTB parsing.
  - [x] `cpio-reader` identified (or custom parser since New ASCII is simple).

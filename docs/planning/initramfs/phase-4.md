# Initramfs - Phase 4: Integration and Testing

## 1. Unit Testing
- [x] Test the CPIO parser with an in-memory byte slice (no boot required).
- [x] Test edge cases for CPIO (empty file, large file, invalid header).

## 2. Integration Testing
- [x] Verify that the initramfs is accessible even if VirtIO Block is disabled.
- [x] Verify that paths with subdirectories are handled correctly.

## 3. Regression Protection
- [x] Ensure that existing FAT32 support is not degraded.
- [x] Confirm no memory leaks or pointers to invalid addresses after MMU re-init.

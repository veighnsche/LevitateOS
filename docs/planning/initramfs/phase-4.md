# Initramfs - Phase 4: Integration and Testing

## 1. Unit Testing
- [ ] Test the CPIO parser with an in-memory byte slice (no boot required).
- [ ] Test edge cases for CPIO (empty file, large file, invalid header).

## 2. Integration Testing
- [ ] Verify that the initramfs is accessible even if VirtIO Block is disabled.
- [ ] Verify that paths with subdirectories are handled correctly.

## 3. Regression Protection
- [ ] Ensure that existing FAT32 support is not degraded.
- [ ] Confirm no memory leaks or pointers to invalid addresses after MMU re-init.

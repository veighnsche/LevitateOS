# TEAM_205: Initramfs VFS Migration

**Objective:** Transition `initramfs` (CPIO) to the VFS architecture and remove legacy dispatch.

## Core Tasks
- [ ] Implement `InitramfsSuperblock` and `InitramfsInodeOps` in `kernel/src/fs/initramfs.rs`
- [ ] Implement CPIO entry to VFS Inode mapping
- [ ] Register initramfs in the VFS mount table during boot
- [ ] Migrate `sys_read` and `sys_getdents` away from legacy initramfs dispatch
- [ ] Remove legacy variants from `FdType`

## Progress Log

### 2026-01-06
- Initialized Phase 14 Step 3 execution.
- Approved implementation plan in `implementation_plan.md`.

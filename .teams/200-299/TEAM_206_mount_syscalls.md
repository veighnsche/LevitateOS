# TEAM_206: Implement Mount/Umount Syscalls (Phase 14 Step 4)

## Objective
Implement `sys_mount` and `sys_umount` syscalls to allow userspace to manage filesystem mounts dynamically, completing the VFS foundational work.

## Checklist

- [ ] Design and Plan [TEAM_206]
    - [ ] Analyze `sys_mount` arguments (source, target, fstype, flags)
    - [ ] Create implementation plan
- [ ] Implementation [TEAM_206]
    - [ ] Implement `sys_mount` in `syscall/fs.rs`
    - [ ] Implement `sys_umount` in `syscall/fs.rs`
    - [ ] Update `kernel/src/fs/mount.rs` to support dynamic operations if needed
    - [ ] Register syscall numbers in `syscall/mod.rs`
- [ ] Verification [TEAM_206]
    - [ ] Verify `mount` failure on non-existent paths
    - [ ] Verify `mount` success (e.g. mounting tmpfs again at /mnt/tmp)
    - [ ] Verify `umount`

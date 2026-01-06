# Team 174: Phase 11 - Core Utilities (The "Busybox" Phase)

## Objective
Implement essential file management and text tools (`ls`, `cat`, `touch`, `mkdir`, `rmdir`, `rm`, `cp`, `mv`, `pwd`, `ln`).

## Progress
- [ ] Research levbox specifications
- [ ] Create implementation plan
- [ ] Implement required kernel syscalls (`getdents`, `unlink`, `mkdir`, `rmdir`, `rename`, etc.)
- [ ] Update `libsyscall` and `ulib`
- [ ] Create `levbox` crate
- [ ] Implement tools in `levbox`
- [ ] Update shell to support command execution with arguments
- [ ] Verify tools in VM

## Notes
- Using a multicall binary approach for `levbox` as hinted by the "Busybox" name.
- Requires updating the shell to parse commands and pass arguments to `sys_spawn`.
- Initramfs is currently read-only; some writable FS support or in-memory overlay might be needed for destructive operations (`mkdir`, `rm`, etc.) if they are to persist or work as expected on the boot disk.

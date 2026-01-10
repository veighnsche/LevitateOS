# Phase 4: Integration and Testing — x86_64 Userspace Compatibility

## Integration Strategy

Once the kernel can dispatch syscalls and userspace can make them, we must verify the full stack.

### Step 1: Build System Updates
- [ ] Update `xtask` to build x86_64 userspace binaries for the initramfs.
- [ ] Ensure `x86_64-unknown-none` target is used for all userspace crates.

### Step 2: Userspace Smoke Test
- [ ] Boot x86_64 kernel with a minimal `init` that only calls `write(1, "Hello x86_64 userspace\n")` and `exit(0)`.
- [ ] Verify output in QEMU serial/VGA.

### Step 3: Full Shell Integration
- [ ] Build `levbox` for x86_64.
- [ ] Boot to shell and verify basic commands (`ls`, `cat`, `echo`).

## Regression Protection
- [ ] Verify AArch64 still builds and boots to shell.
- [ ] Ensure no architecture-specific leakage in common VFS/Process code.

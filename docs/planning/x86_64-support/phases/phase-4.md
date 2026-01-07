# Phase 4: Integration and Testing — x86_64 Support

## Integration Strategy

Once the kernel reaches `kernel_main` and has basic HAL support, we must ensure it integrates with the rest of the OS.

> **Note**: Each step has a detailed breakdown in its own file with SLM-sized units of work.

### Step 1: Userspace x86_64 Compatibility
**Detailed Plan**: [phase-4-step-1.md](phase-4-step-1.md) — 6 UoWs

- [ ] UoW 1.1: Add x86_64 Syscall Entry to libsyscall
- [ ] UoW 1.2: Add x86_64 Module to libsyscall
- [ ] UoW 1.3: Port ulib Entry Point for x86_64
- [ ] UoW 1.4: Verify Syscall Dispatch in Kernel
- [ ] UoW 1.5: Build levbox for x86_64
- [ ] UoW 1.6: Port init Binary

### Step 2: Initramfs and VFS Integration
**Detailed Plan**: [phase-4-step-2.md](phase-4-step-2.md) — 4 UoWs

- [ ] UoW 2.1: Parse Multiboot2 Module Tags
- [ ] UoW 2.2: Map Initramfs into Virtual Memory
- [ ] UoW 2.3: Verify CPIO Parser Compatibility
- [ ] UoW 2.4: Wire Initramfs to VFS on x86_64

### Step 3: Testing and Regression Protection
**Detailed Plan**: [phase-4-step-3.md](phase-4-step-3.md) — 6 UoWs

- [ ] UoW 3.1: Add x86_64 Build Verification Test
- [ ] UoW 3.2: Add x86_64 Boot Test
- [ ] UoW 3.3: Add x86_64 Userspace Smoke Test
- [ ] UoW 3.4: Create x86_64 Behavior Golden File
- [ ] UoW 3.5: Verify AArch64 Tests Still Pass
- [ ] UoW 3.6: Add CI Matrix for Both Architectures

## Regression Protection
- [ ] Existing AArch64 tests must continue to pass.
- [ ] CI should run both AArch64 and x86_64 builds.

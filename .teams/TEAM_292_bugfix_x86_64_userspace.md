# TEAM_292: x86_64 Userspace Build Bugfix

## 1. Team Registration
- **Team ID**: TEAM_292
- **Predecessor**: TEAM_289 (x86_64 GPF investigation)
- **Focus**: Fix x86_64 init spawn failure - build userspace for x86_64

## 2. Bug Report

### Symptom
```
[BOOT] ERROR: Failed to spawn init: Elf(NotExecutable)
```

### Root Cause
- Initramfs contains aarch64 ELF binaries (`e_machine = 183`)
- x86_64 kernel expects `e_machine = 62` (EM_X86_64)
- ELF loader correctly rejects wrong architecture at `elf.rs:170`

### Environment
- Architecture: x86_64
- Userspace target: currently only aarch64-unknown-none

## 3. Investigation Summary

ELF loader validates machine type:
```rust
// kernel/src/loader/elf.rs:170
if header.e_machine != crate::arch::ELF_MACHINE {
    return Err(ElfError::WrongArchitecture);
}
```

Error returns `NotExecutable` instead of `WrongArchitecture` because check at line 166 fails first (ET_EXEC check) - but actual failure is architecture mismatch.

## 4. Fix Scope

Need to build userspace for x86_64:
1. Update xtask build to support multi-arch userspace
2. Compile levbox/init/shell for x86_64-unknown-none
3. Create x86_64 initramfs with correct binaries
4. Include in ISO build

## 5. Planning Reference
See implementation_plan.md for detailed phases.

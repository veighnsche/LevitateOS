# Eyra Userspace Behavior Inventory

This document catalogs all testable behaviors in the Eyra userspace integration.

## Behavior ID Prefix: EY (Eyra)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| **Build System** |
| EY1 | libsyscall builds for aarch64-unknown-linux-gnu with std feature | ✅ | `test_libsyscall_builds_aarch64` |
| EY2 | libsyscall-tests binary produces 64-bit ARM ELF | ✅ | `test_libsyscall_tests_elf_format` |
| EY3 | Binary is statically linked (no dynamic dependencies) | ✅ | `test_binary_static_linkage` |
| EY4 | Binary size is reasonable (<100KB for tests) | ✅ | `test_binary_size_limit` |
| EY5 | Cross-compilation uses correct sysroot path | ✅ | `test_sysroot_configuration` |
| **Linker Configuration** |
| EY6 | Workspace .cargo/config.toml contains -nostartfiles | ✅ | `test_nostartfiles_in_workspace_config` |
| EY7 | -nostartfiles is NOT duplicated in build.rs files | ✅ | `test_no_duplicate_nostartfiles` |
| EY8 | aarch64 config includes correct sysroot path | ✅ | `test_aarch64_sysroot_in_config` |
| EY9 | All targets use +crt-static | ✅ | `test_crt_static_enabled` |
| EY10 | All targets use relocation-model=pic | ✅ | `test_pic_relocation_model` |
| **Build Artifacts** |
| EY11 | libgcc_eh.a stub is created during build | ✅ | `test_libgcc_eh_stub_exists` |
| EY12 | getauxval stub is compiled and linked | ✅ | `test_getauxval_stub_linked` |
| EY13 | Build succeeds without libgcc_eh errors | ✅ | `test_no_libgcc_eh_errors` |
| **Binary Properties** |
| EY14 | Entry point is in valid code range (0x400000+) | ✅ | `test_entry_point_valid` |
| EY15 | Binary has LOAD segments at expected addresses | ✅ | `test_load_segments_addresses` |
| EY16 | Text segment has R-X permissions | ✅ | `test_text_segment_permissions` |
| EY17 | Data segment has RW permissions | ✅ | `test_data_segment_permissions` |
| EY18 | Binary has no INTERP segment (static) | ✅ | `test_no_interp_segment` |
| **Dependencies** |
| EY19 | libsyscall depends on linux-raw-sys 0.4 | ✅ | `test_linux_raw_sys_version` |
| EY20 | eyra dependency is marked optional | ✅ | `test_eyra_dependency_optional` |
| EY21 | std feature enables eyra dependency | ✅ | `test_std_feature_enables_eyra` |
| EY22 | Default features do not include std | ✅ | `test_default_no_std` |
| **Cargo Workspace** |
| EY23 | eyra/Cargo.toml includes libsyscall member | ✅ | `test_libsyscall_in_workspace` |
| EY24 | eyra/Cargo.toml includes libsyscall-tests member | ✅ | `test_libsyscall_tests_in_workspace` |
| EY25 | Workspace resolver is set to "2" | ✅ | `test_workspace_resolver_version` |
| **Documentation** |
| EY26 | NOSTARTFILES_README.md exists | ✅ | `test_nostartfiles_readme_exists` |
| EY27 | X86_64_STATUS.md documents limitation | ✅ | `test_x86_64_status_documented` |
| EY28 | TEAM_380 documents cross-compilation setup | ✅ | `test_team_380_exists` |
| EY29 | TEAM_381 documents nostartfiles abstraction | ✅ | `test_team_381_exists` |
| EY30 | TEAM_382 documents integration test results | ✅ | `test_team_382_exists` |
| **Initramfs Integration** |
| EY31 | libsyscall-tests can be added to initramfs | ✅ | `test_binary_in_initramfs` |
| EY32 | Binary is executable in initramfs (permissions) | ✅ | `test_binary_executable_perms` |
| EY33 | Initramfs includes exactly 30 binaries with libsyscall-tests | ✅ | `test_initramfs_count` |
| **Known Issues** |
| EY34 | x86_64 build fails with std conflicts (documented) | ✅ | `test_x86_64_build_fails_expected` |
| EY35 | Binary spawns successfully on LevitateOS (PID assigned) | ⚠️ | `test_binary_spawns_on_levitate` |
| EY36 | Binary execution crashes at address 0x0 (kernel bug) | ⚠️ | `test_execution_crash_documented` |

## Behavior ID Prefix: LS (LibSyscall)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| **Syscall Wrappers** |
| LS1 | sys_write wraps linux write syscall | ✅ | `test_sys_write_wraps_syscall` |
| LS2 | sys_read wraps linux read syscall | ✅ | `test_sys_read_wraps_syscall` |
| LS3 | sys_open wraps linux openat syscall | ✅ | `test_sys_open_wraps_openat` |
| LS4 | sys_close wraps linux close syscall | ✅ | `test_sys_close_wraps_syscall` |
| **Architecture Abstraction** |
| LS5 | aarch64 uses x8 for syscall number | ✅ | `test_aarch64_syscall_convention` |
| LS6 | aarch64 uses x0-x5 for arguments | ✅ | `test_aarch64_argument_registers` |
| LS7 | aarch64 uses svc #0 instruction | ✅ | `test_aarch64_svc_instruction` |
| LS8 | x86_64 uses rax for syscall number | ✅ | `test_x86_64_syscall_convention` |
| LS9 | x86_64 uses rdi,rsi,rdx,r10,r8,r9 for args | ✅ | `test_x86_64_argument_registers` |
| **Error Handling** |
| LS10 | Negative return values indicate errors | ✅ | `test_negative_return_is_error` |
| LS11 | Errors are returned as-is (no errno mapping) | ✅ | `test_no_errno_translation` |
| LS12 | Success returns non-negative values | ✅ | `test_success_nonnegative` |
| **Const Safety** |
| LS13 | All syscall numbers are const | ✅ | `test_syscall_numbers_const` |
| LS14 | AT_FDCWD is const and correct value | ✅ | `test_at_fdcwd_const` |
| LS15 | O_RDONLY/O_WRONLY/O_RDWR are const | ✅ | `test_open_flags_const` |

## Test Status Legend

- ✅ Tested with traceability
- ⚠️ Partially tested (known limitations)
- ❌ Not yet tested
- 🚧 Test in progress

## Next Steps

1. Add unit tests for all LS behaviors (libsyscall core functionality)
2. Create behavior test for full spawn-and-execute cycle (blocked on kernel bug)
3. Add regression test to verify no `_start` symbol conflicts
4. Create performance benchmarks for syscall overhead

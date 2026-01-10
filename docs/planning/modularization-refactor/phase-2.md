# Phase 2: Structural Extraction - Modularization Refactor

**TEAM_372: Defining the new module structure.**

## Target Design

### HAL GIC (aarch64)
```
crates/hal/src/aarch64/gic/
├── mod.rs        # Gic struct, API/API_V3 statics, re-exports
├── distributor.rs # Register offsets, GicdCtlrFlags, detect_gic_version
├── handlers.rs    # HANDLERS array, register_handler, dispatch
├── v2.rs         # Gic::init_v2, GICv2 specific methods
└── v3.rs         # Gic::init_v3, sysreg mod, GICv3 specific methods
```

### Kernel Init
```
crates/kernel/src/init/
├── mod.rs             # run() sequence
├── boot_stage.rs      # BootStage enum, transition_to()
├── failsafe_shell.rs  # maintenance_shell()
├── device_discovery.rs # Unified device discovery (init_devices, init_display)
├── fs_mount.rs        # mount_tmpfs_at_dentry, init_filesystem
└── userspace.rs       # init_userspace, spawn_init
```

### Kernel Syscall Process
```
crates/kernel/src/syscall/process/
├── mod.rs        # sys_exit, sys_getpid, sys_yield, etc.
├── spawn.rs      # sys_spawn, sys_spawn_args, sys_exec, UserArgvEntry
├── waitpid.rs    # sys_waitpid, reap_zombie integration
├── thread.rs     # sys_clone, sys_set_tid_address, sys_gettid
└── arch_prctl.rs # sys_arch_prctl (x86_64 specific logic)
```

### Subsystems Organization
```
crates/kernel/src/subsystems/
├── mod.rs        # Subsystem management
├── gpu.rs        # Moved from kernel/src/gpu.rs
├── input.rs      # Moved from kernel/src/input.rs
├── net.rs        # Moved from kernel/src/net.rs
├── block.rs      # Moved from kernel/src/block.rs
└── virtio.rs     # Moved from kernel/src/virtio.rs
```

## Extraction Strategy
1. Create new directories and `mod.rs` files.
2. Move logic block by block, ensuring compilation at each step.
3. Update imports in `kernel/src/lib.rs` and other modules.
4. Clean up any dead imports or re-exports.

---

## Steps and UoWs
Refer to `phase-2-step-N.md` for detailed tasks.

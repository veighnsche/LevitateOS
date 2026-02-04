# AcornOS Mirroring Checklist

> **Goal:** Extract reusable code from leviso to distro-builder to give AcornOS better shared infrastructure support.

---

## Phase 1: Component Executor Operations (Immediate)

These are distro-agnostic operations that AcornOS has reimplemented inline.

### 1.1 Directory Operations
- [ ] Extract `leviso/src/component/executor/directories.rs` to `distro-builder/src/component/executor/directories.rs`
  - [ ] `handle_dir()` - Create directory
  - [ ] `handle_dirmode()` - Create directory with permissions
  - [ ] `handle_dirs()` - Create multiple directories
  - [ ] Include tests
- [ ] Update `distro-builder/src/component/mod.rs` to re-export
- [ ] Update AcornOS to use `distro_builder::component::executor::directories::*`
- [ ] Remove duplicate code from `AcornOS/src/component/executor.rs` (lines 34-48)

### 1.2 File Operations
- [ ] Extract `leviso/src/component/executor/files.rs` to `distro-builder/src/component/executor/files.rs`
  - [ ] `handle_copyfile()` - Copy file from source
  - [ ] `handle_copytree()` - Copy directory tree
  - [ ] `handle_writefile()` - Write file with content
  - [ ] `handle_writefilemode()` - Write file with permissions
  - [ ] `handle_symlink()` - Create symlink
  - [ ] Include tests
- [ ] Update `distro-builder/src/component/mod.rs` to re-export
- [ ] Update AcornOS to use `distro_builder::component::executor::files::*`
- [ ] Remove duplicate code from `AcornOS/src/component/executor.rs` (lines 53-99)

### 1.3 User/Group Operations
- [ ] Extract `leviso/src/component/executor/users.rs` to `distro-builder/src/component/executor/users.rs`
  - [ ] `handle_user()` - Create/update user
  - [ ] `handle_group()` - Create/update group
  - [ ] Include tests
- [ ] Update `distro-builder/src/component/mod.rs` to re-export
- [ ] Update AcornOS to use `distro_builder::component::executor::users::*`
- [ ] Remove duplicate code from `AcornOS/src/component/executor.rs` (lines 160-172)

**Phase 1 Result:** AcornOS executor.rs reduced from ~200 lines to ~50 lines (OpenRC + Custom only)

---

## Phase 2: Build Utilities (High Priority)

### 2.1 User/Group File Management
- [ ] Extract `leviso/src/build/users.rs` to `distro-builder/src/build/users.rs`
  - [ ] `read_uid_from_rootfs()` - Parse passwd file
  - [ ] `read_gid_from_rootfs()` - Parse group file
  - [ ] `ensure_user()` - Add user with proper formatting
  - [ ] `ensure_group()` - Add group with proper formatting
  - [ ] Include tests
- [ ] Update `distro-builder/src/build/mod.rs` to re-export
- [ ] Update AcornOS executor to call `distro_builder::build::users::ensure_user()`
- [ ] Remove `ensure_user()` and `ensure_group()` from `AcornOS/src/component/executor.rs`

### 2.2 Filesystem Structure Creation
- [ ] Extract `leviso/src/build/filesystem.rs` to `distro-builder/src/build/filesystem.rs`
  - [ ] `create_fhs_structure()` - Parameterize library path
  - [ ] `create_var_symlinks()` - Create /var/run, /var/lock
  - [ ] Add `lib_dir` parameter ("usr/lib" for musl, "usr/lib64" for glibc)
  - [ ] Include tests
- [ ] Update `distro-builder/src/build/mod.rs` to re-export
- [ ] Update AcornOS `component/custom/filesystem.rs` to use shared version
- [ ] Remove `create_fhs_symlinks()` from `AcornOS/src/component/custom/filesystem.rs`

### 2.3 Build Context Trait Extensions
- [ ] Extend `distro-builder/src/build/context.rs` trait
  - [ ] Add `lib_path(&self) -> &'static str` method
  - [ ] Add `find_binary(&self, name: &str) -> Option<PathBuf>` method
  - [ ] Add `source_exists(&self, path: &str) -> bool` method
- [ ] Update `AcornOS/src/component/context.rs` to implement new methods
- [ ] Update `leviso/src/build/context.rs` to implement new methods
- [ ] Ensure both implementations are consistent

---

## Phase 3: Common Utilities (Medium Priority)

### 3.1 Path Utilities
- [ ] Extract `leviso/src/common/paths.rs` to `distro-builder/src/util/paths.rs`
  - [ ] `find_and_copy_dir()` - Find directory from primary/fallback
  - [ ] `find_dir()` - Find directory from multiple locations
  - [ ] `ensure_dir_exists()` - Create directory if missing
  - [ ] `ensure_parent_exists()` - Create parent directories
- [ ] Create `distro-builder/src/util/mod.rs` to expose utilities
- [ ] Update `distro-builder/src/lib.rs` to re-export
- [ ] Update leviso to use `distro_builder::util::paths::*`
- [ ] Update AcornOS to use `distro_builder::util::paths::*`

### 3.2 Temporary Directory Utilities
- [ ] Extract `leviso/src/common/temp.rs` to `distro-builder/src/util/temp.rs`
  - [ ] `prepare_work_dir()` - Create fresh work directory
  - [ ] `cleanup_work_dir()` - Remove work directory
- [ ] Update `distro-builder/src/util/mod.rs` to re-export
- [ ] Update leviso to use `distro_builder::util::temp::*`
- [ ] Update AcornOS to use `distro_builder::util::temp::*`

### 3.3 File Utilities
- [ ] Extract `leviso/src/common/files.rs` to `distro-builder/src/util/files.rs`
  - [ ] `write_file_with_dirs()` - Write file, creating parent dirs
  - [ ] `write_file_mode()` - Write file with specific permissions
- [ ] Update `distro-builder/src/util/mod.rs` to re-export
- [ ] Update leviso to use `distro_builder::util::files::*`
- [ ] Update AcornOS to use `distro_builder::util::files::*`

---

## Phase 4: Preflight Framework (Medium Priority)

### 4.1 Check Result Types
- [ ] Extract `leviso/src/preflight/types.rs` to `distro-builder/src/preflight/types.rs`
  - [ ] `CheckResult` struct with pass/fail/warn constructors
  - [ ] `PreflightReport` struct with aggregation methods
  - [ ] Include display/formatting logic
- [ ] Update `distro-builder/src/preflight/mod.rs` to re-export
- [ ] Update leviso preflight to use shared types
- [ ] Update AcornOS preflight to use shared types
- [ ] Remove `CheckResult` from `AcornOS/src/preflight/mod.rs`

### 4.2 Host Tool Checking
- [ ] Extend `distro-builder/src/preflight/mod.rs`
  - [ ] Add `check_tool_exists()` function
  - [ ] Add `check_required_tools()` function
  - [ ] Make tool list configurable per-distro
- [ ] Update leviso preflight to use shared checking
- [ ] Update AcornOS preflight to use shared checking

---

## Phase 5: Component System Architecture (Long-term)

### 5.1 Generic Executor Dispatcher
- [ ] Design trait-based executor in `distro-builder`
  - [ ] `GenericExecutor` trait with `execute_op()` method
  - [ ] `ExecutorContext` for shared state
  - [ ] Distro-specific extension points
- [ ] Implement generic handlers for standard operations
- [ ] Create extension mechanism for distro-specific ops (OpenRC, systemd)

### 5.2 Component Builder Orchestration
- [ ] Extract shared `build_system()` pattern
  - [ ] Design `ComponentOrchestrator` trait
  - [ ] Support phase ordering
  - [ ] Support pre/post hooks
- [ ] Update leviso to use orchestrator
- [ ] Update AcornOS to use orchestrator

### 5.3 Operation Enum Unification
- [ ] Design unified `Op` enum in `distro-builder`
  - [ ] Generic operations: Dir, File, Symlink, User, Group
  - [ ] Extension point for init-system operations
  - [ ] Extension point for custom operations
- [ ] Update `distro-builder::component::Op`
- [ ] Update leviso to use unified enum
- [ ] Update AcornOS to use unified enum

---

## Phase 6: Testing & Validation

### 6.1 Unit Tests
- [ ] Ensure all extracted modules have tests
- [ ] Run `cargo test` in distro-builder
- [ ] Verify tests pass on both glibc and musl hosts

### 6.2 Integration Tests
- [ ] Test leviso build still works
- [ ] Test AcornOS build still works
- [ ] Verify no regressions in output

### 6.3 Documentation
- [ ] Update `distro-builder/README.md` with new modules
- [ ] Add rustdoc to all public APIs
- [ ] Create migration guide for distro implementations

---

## Code Reduction Targets

| File | Current Lines | Target Lines | Reduction |
|------|---------------|--------------|-----------|
| `AcornOS/src/component/executor.rs` | ~200 | ~50 | 75% |
| `AcornOS/src/component/custom/filesystem.rs` | ~100 | ~30 | 70% |
| `AcornOS/src/preflight/mod.rs` | ~150 | ~80 | 45% |
| `AcornOS/src/component/context.rs` | ~120 | ~80 | 33% |
| **Total AcornOS reduction** | **~570** | **~240** | **~330 lines** |

---

## Dependency Updates

### distro-builder/Cargo.toml
```toml
[dependencies]
# Existing...

# Add if needed for extracted code
walkdir = "2"  # Already present
```

### leviso/Cargo.toml
```toml
[dependencies]
# Update to use new distro-builder features
distro-builder = { path = "../distro-builder" }
```

### AcornOS/Cargo.toml
```toml
[dependencies]
# Update to use new distro-builder features
distro-builder = { path = "../distro-builder" }
```

---

## Progress Tracking

| Phase | Status | Completion % |
|-------|--------|--------------|
| Phase 1: Executor Operations | Not started | 0% |
| Phase 2: Build Utilities | Not started | 0% |
| Phase 3: Common Utilities | Not started | 0% |
| Phase 4: Preflight Framework | Not started | 0% |
| Phase 5: Architecture | Not started | 0% |
| Phase 6: Testing | Not started | 0% |

---

## Notes

- **Priority order:** Phase 1 > Phase 2 > Phase 3 > Phase 4 > Phase 5
- **Testing:** Each phase should include tests before moving to next
- **Backward compatibility:** Maintain leviso functionality throughout
- **AcornOS goal:** Focus only on OpenRC-specific code, delegate generic operations to distro-builder

---

## Related Files

### Source (leviso)
- `leviso/src/component/executor/directories.rs`
- `leviso/src/component/executor/files.rs`
- `leviso/src/component/executor/users.rs`
- `leviso/src/build/users.rs`
- `leviso/src/build/filesystem.rs`
- `leviso/src/common/paths.rs`
- `leviso/src/common/temp.rs`
- `leviso/src/common/files.rs`
- `leviso/src/preflight/types.rs`

### Target (distro-builder)
- `distro-builder/src/component/executor/` (new directory)
- `distro-builder/src/build/users.rs` (new file)
- `distro-builder/src/build/filesystem.rs` (new file)
- `distro-builder/src/util/` (new directory)
- `distro-builder/src/preflight/types.rs` (new file)

### Consumer (AcornOS)
- `AcornOS/src/component/executor.rs` (reduce significantly)
- `AcornOS/src/component/custom/filesystem.rs` (reduce)
- `AcornOS/src/preflight/mod.rs` (reduce)
- `AcornOS/src/component/context.rs` (implement trait extensions)

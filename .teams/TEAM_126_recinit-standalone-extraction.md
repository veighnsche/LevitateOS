# TEAM_126: Extract Initramfs Builder into Standalone Repository (recinit)

## Status: COMPLETE

## Goal

Extract the initramfs building code into a standalone repository that can be:
1. Used independently as a CLI tool (`recinit`)
2. Used as a library by leviso (via git dependency or path)
3. Potentially used by AcornOS or other projects

## Repository Location

**GitHub:** https://github.com/LevitateOS/recinit

## Repository Structure

```
tools/recinit/
├── Cargo.toml           # Standalone workspace
├── README.md            # Full documentation
├── LICENSE-MIT
├── LICENSE-APACHE
├── src/
│   ├── main.rs          # CLI entry point (clap-based)
│   ├── lib.rs           # Public API: InitramfsBuilder
│   ├── tiny.rs          # build_tiny_initramfs logic
│   ├── install.rs       # build_install_initramfs logic
│   ├── busybox.rs       # Busybox setup
│   ├── modules.rs       # Kernel module presets (Live, Install, Custom)
│   ├── systemd.rs       # Systemd/firmware copying
│   ├── cpio.rs          # Pure Rust CPIO building (no shell)
│   └── elf.rs           # ELF dependency analysis
├── templates/
│   └── init_tiny.template
└── tests/               # 13 unit tests
```

## Dependencies Extracted

| Original Source | New Location |
|-----------------|--------------|
| `distro-builder::build_cpio` | `recinit/src/cpio.rs` (rewritten in pure Rust) |
| `leviso-elf::*` | `recinit/src/elf.rs` |
| `leviso::initramfs.rs` | `recinit/src/{tiny,install,busybox,modules,systemd}.rs` |
| `leviso/profile/init_tiny.template` | `recinit/templates/init_tiny.template` |

## Key Design Decisions

1. **No monorepo dependencies** - recinit is fully standalone with `[workspace]` in Cargo.toml
2. **Pure Rust CPIO** - No shell commands, writes CPIO newc format directly
3. **Module presets** - `ModulePreset::Live`, `ModulePreset::Install`, `ModulePreset::Custom(Vec<String>)`
4. **Dual CLI/Library** - Works as both `recinit` binary and `recinit` crate
5. **Template variables** - `{{ISO_LABEL}}`, `{{ROOTFS_PATH}}`, `{{BOOT_MODULES}}`, `{{BOOT_DEVICES}}`, `{{LIVE_OVERLAY_PATH}}`

## CLI Usage

```bash
# Build live initramfs
recinit build-tiny \
  --modules-dir /lib/modules/$(uname -r) \
  --busybox /usr/bin/busybox \
  --template templates/init_tiny.template \
  --output initramfs.cpio.gz \
  --iso-label LEVITATEOS

# Build install initramfs
recinit build-install \
  --rootfs /path/to/rootfs-staging \
  --output initramfs-installed.img

# List module presets
recinit modules --list-presets
```

## Library Usage

```rust
use recinit::{InitramfsBuilder, TinyConfig, InstallConfig, ModulePreset};

let builder = InitramfsBuilder::new();
builder.build_tiny(TinyConfig {
    modules_dir: PathBuf::from("/lib/modules/6.12.0"),
    busybox_path: PathBuf::from("/usr/bin/busybox"),
    // ...
})?;
```

## Integration with leviso

To use recinit from leviso, add to `leviso/Cargo.toml`:

**Option A: Path dependency (current)**
```toml
[dependencies]
recinit = { path = "../tools/recinit" }
```

**Option B: Git dependency (after pushing to GitHub)**
```toml
[dependencies]
recinit = { git = "https://github.com/LevitateOS/recinit" }
```

## Test Results

```
running 13 tests
test busybox::tests::test_busybox_commands_not_empty ... ok
test cpio::tests::test_build_cpio ... ok
test cpio::tests::test_build_cpio_with_symlink ... ok
test elf::tests::test_parse_readelf_empty ... ok
test elf::tests::test_parse_readelf_output ... ok
test install::tests::test_default_config ... ok
test modules::tests::test_install_modules_superset_of_storage ... ok
test modules::tests::test_live_modules_includes_essentials ... ok
test modules::tests::test_module_name ... ok
test modules::tests::test_preset_from_str ... ok
test tiny::tests::test_default_boot_devices ... ok
test tiny::tests::test_default_config ... ok
test busybox::tests::test_setup_busybox_creates_symlinks ... ok

test result: ok. 13 passed; 0 failed
```

## Progress

- [x] Analyze existing code in leviso, distro-builder, leviso-elf
- [x] Create TEAM_126 file
- [x] Create recinit directory structure
- [x] Implement src/cpio.rs (pure Rust CPIO writer)
- [x] Implement src/elf.rs (from leviso-elf)
- [x] Implement src/modules.rs with presets
- [x] Implement src/busybox.rs
- [x] Implement src/tiny.rs
- [x] Implement src/install.rs
- [x] Implement src/systemd.rs
- [x] Implement src/lib.rs (public API)
- [x] Implement src/main.rs (CLI with clap)
- [x] Copy init_tiny.template
- [x] Create README.md
- [x] Add LICENSE-MIT and LICENSE-APACHE
- [x] Add tests (13 tests)
- [x] Verify standalone build (`cargo build` succeeds)
- [x] Verify tests pass (`cargo test` - 13/13 passed)
- [x] Verify CLI works (`recinit --help`)

## Monorepo Integration (2026-01-27)

1. [x] Added recinit as git submodule at `tools/recinit`
2. [x] Updated `leviso/Cargo.toml` with `recinit = { path = "../tools/recinit" }`
3. [x] Replaced 833-line `leviso/src/artifact/initramfs.rs` with ~170-line wrapper
   - Removed all direct CPIO, busybox, module, systemd copying code
   - Now just configures `TinyConfig` and `InstallConfig` and calls recinit
   - Retained leviso-specific logic: finding modules in staging vs downloads
4. [x] Build verified: `cargo build` succeeds in leviso

## Code Reduction

| Before | After | Reduction |
|--------|-------|-----------|
| 833 lines | ~170 lines | **80% reduction** |

The initramfs.rs now only contains:
- `build_tiny_initramfs(base_dir)` - configures TinyConfig, calls recinit
- `build_install_initramfs(base_dir)` - configures InstallConfig, calls recinit
- `find_kernel_modules_dir(base_dir)` - leviso-specific module path logic
- `find_kernel_version(modules_dir)` - helper for kernel version detection

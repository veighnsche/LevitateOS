# LevitateOS Behaviors Inventory

This document tracks all observable behaviors and invariants in the LevitateOS codebase.

## Table of Contents

- [Build System Behaviors](#build-system-behaviors)
- [Initialization & Boot Behaviors](#initialization--boot-behaviors)
- [QEMU Integration Behaviors](#qemu-integration-behaviors)
- [Test Behaviors](#test-behaviors)
- [VM Session Behaviors](#vm-session-behaviors)
- [Error Handling Behaviors](#error-handling-behaviors)
- [File System Behaviors](#file-system-behaviors)

---

## Build System Behaviors

### Linux Kernel Build
- **Description**: Builds Linux kernel from submodule source
- **Trigger**: `cargo run -- build linux` or as part of `build all`
- **Input**: `linux/` submodule at specific version
- **Output**: Architecture-specific kernel binary
  - `linux/arch/x86_64/boot/bzImage` (x86_64)
  - `linux/arch/arm64/boot/Image` (aarch64)
- **Side Effects**: Modifies `linux/` build artifacts
- **Error Handling**: Fails with clear error if source missing

### BusyBox Build
- **Description**: Builds BusyBox shell + utilities with musl static linking
- **Trigger**: `cargo run -- build busybox` or as part of `build all`
- **Output**: Architecture-specific binary at `toolchain/busybox-out/{arch}/busybox`
- **Guarantees**: Statically linked with musl libc
- **Side Effects**: Creates `toolchain/busybox-out/` directory
- **Architecture Support**: x86_64, aarch64

### OpenRC Build
- **Description**: Builds OpenRC init system from source with meson/ninja
- **Trigger**: `cargo run -- build openrc` or as part of `build all`
- **Requirements**: meson, ninja, musl-gcc on system
- **Version**: 0.54 (cloned from GitHub)
- **Output**: `toolchain/openrc-out/{arch}/`
- **Binary Locations**:
  - `sbin/openrc`
  - `sbin/openrc-run`
  - `sbin/rc-service`
  - `sbin/rc-update`
  - `bin/rc-status`
- **Verification**: Checks binaries exist and are statically linked
- **Cross-Compilation**: Uses musl-cross.txt configuration file

### Initramfs Build
- **Description**: Assembles initramfs CPIO archive from declarative manifest
- **Trigger**: `cargo run -- build initramfs` or automatically before `run`
- **Input**: `initramfs/initramfs.toml` manifest file
- **Output**: `target/initramfs/{arch}.cpio`
- **Legacy Output**: `initramfs_{arch}.cpio` (for compatibility)
- **Manifest Processing**:
  - Parses TOML file into directory/file structure
  - Validates file paths and permissions
  - Generates CPIO archive with TUI progress (if interactive)
- **Dependencies**: Requires BusyBox and OpenRC to be built first
- **Size**: ~30-50 MB gzipped depending on arch

---

## Initialization & Boot Behaviors

### Preflight Checks
- **Trigger**: Automatic before `build`, `run`, or `test` commands
- **Checks**:
  - System tools: cargo, rustup, find, cpio, dd, curl, git
  - Architecture-specific tools (xorriso for x86_64)
  - Rust targets installed (`x86_64-unknown-none`, `aarch64-unknown-none`)
  - `rust-src` component available
- **Failure Behavior**: Prints specific installation instructions and exits
- **Arch-Specific**: Different tool lists for x86_64 vs aarch64

### Project Root Detection
- **Behavior**: Automatically locates project root using CARGO_MANIFEST_DIR
- **Fallback**: Falls back to current directory if env var not set
- **Special Case**: Handles `xtask/` subdirectory by going up one level
- **Effect**: Sets current directory to project root before any operation

### Initramfs Assembly
- **Sequence**:
  1. Verify BusyBox binary exists
  2. Verify OpenRC binaries exist
  3. Load and parse `initramfs/initramfs.toml`
  4. Build CPIO archive with all specified files
  5. Write to `target/initramfs/{arch}.cpio`
  6. Copy to `initramfs_{arch}.cpio` for legacy compatibility
- **Error Stops**: Fails immediately if any required binary missing

---

## QEMU Integration Behaviors

### Profile-Based Configuration
- **Default Profile**: 2GB RAM, 1 core, cortex-a53 (aarch64) or qemu64 (x86_64)
- **Pixel6 Profile**: 8GB RAM, 8 cores, cortex-a76, GICv3
- **GicV3 Profile**: 2GB RAM, 1 core with GICv3
- **X86_64 Profile**: q35 machine, qemu64 CPU, 2GB RAM

### Boot Configuration
- **Always Uses**: Linux kernel from source (not custom kernel)
- **Kernel Path**: `linux/arch/{arch}/boot/{bzImage|Image}`
- **Initramfs Path**: `target/initramfs/{arch}.cpio`
- **Kernel Command Line**: `console=ttyS0 earlyprintk=serial,ttyS0,115200 rdinit=/init`
- **No Reboot**: `-no-reboot` flag always set (VM exits instead of rebooting)

### Display Modes
- **GTK/SDL**: Uses `-vga std` for Limine framebuffer
- **VNC**: Uses virtio-gpu with proper resolution via EDID
- **Headless**: Display disabled, serial to stdio or file
- **Nographic**: No VGA, simple serial on stdio (for tests)

### GPU Configuration
- **x86_64 GTK**: VGA std (default)
- **x86_64 VNC/Headless**: virtio-gpu-pci with EDID
- **aarch64**: Always virtio-gpu-pci
- **Resolution**: 1280x800 by default (128x36 character grid)

### Input Devices
- **Normal Modes**: virtio-keyboard and virtio-tablet
- **Nographic Mode**: No input devices (uses serial only)
- **Architecture**: Uses device suffix (aarch64) or pci (x86_64)

### Network
- **Config**: User-mode networking with virtio-net
- **Network ID**: `net0` with `user` backend

### QMP Socket
- **Default Location**: `./qmp.sock`
- **Session Location**: `./qemu-session.sock`
- **Creation**: Only created if `enable_qmp()` called
- **Cleanup**: Removed on QEMU exit or `vm stop`

### GDB Support
- **Port**: 1234 (standard QEMU GDB port)
- **Wait Mode**: `-S` flag freezes CPU at startup until GDB connects
- **Usage**: For kernel debugging during boot

---

## Test Behaviors

### Unit Tests
- **Trigger**: `cargo run -- test unit`
- **Scope**: Rust unit tests in crate (19 tests)
- **No VM**: Tests run host-side without booting
- **Areas Covered**:
  - CPIO archive generation
  - Manifest parsing
  - QEMU command building
  - Architecture enum parsing
  - Configuration loading

### Behavior Tests (Golden File)
- **Trigger**: `cargo run -- test behavior`
- **Arch**: aarch64 default, x86_64 with special handling
- **Input**: Boot output from QEMU
- **Golden File**: `tests/golden_boot_linux_openrc.txt`
- **Comparison**: Line-by-line diff (with configurable tolerance)
- **Update Mode**: `--update` flag refreshes golden file
- **Rating**: Gold by default (fails on mismatch), can be Silver (auto-update)

### Serial Input Tests
- **Trigger**: `cargo run -- test serial`
- **Input Method**: Raw serial bytes sent to VM stdin
- **Commands Tested**: Basic shell commands via serial
- **Arch**: x86_64 only (uses disk image)

### Keyboard Input Tests
- **Trigger**: `cargo run -- test keyboard`
- **Input Method**: QMP sendkey commands
- **Test Case**: Typing text and verifying shell echoes correctly
- **Arch**: x86_64 with disk image

### Shutdown Tests
- **Trigger**: `cargo run -- test shutdown`
- **Verification**: Checks for clean shutdown via serial output
- **Timeout**: 30 seconds default

### Screenshot Tests
- **Subtypes**:
  - `screenshot`: All screenshot tests
  - `screenshot:alpine`: External Alpine Linux screenshots
  - `screenshot:levitate`: LevitateOS boot screenshots
- **Format**: PPM converted to PNG if `magick` available
- **Output**: `tests/screenshots/`

### Debug Tools
- **Trigger**: `cargo run -- test debug`
- **Purpose**: Development utilities for examining golden files
- **Features**: Diff display, formatting inspection

---

## VM Session Behaviors

### VM Start
- **Trigger**: `cargo run -- vm start`
- **Persistence**: Saves session state to `.qemu-session.json`
- **Background**: Runs QEMU in background with VNC display
- **Socket**: Creates `./qemu-session.sock` for QMP
- **Validation**: Checks PID is alive before reusing old session
- **Stale Cleanup**: Auto-removes dead sessions

### VM Stop
- **Trigger**: `cargo run -- vm stop`
- **Action**: Kills QEMU process by PID
- **Cleanup**: Removes `.qemu-session.json` and QMP socket
- **Idempotent**: Safe to run even if no session

### VM Send
- **Trigger**: `cargo run -- vm send "text"`
- **Method**: QMP sendkey commands via char-to-qcode mapping
- **Enter**: Automatically appended after text
- **Delay**: 50ms between characters
- **Validation**: Requires active session (checks PID alive)

### VM Screenshot
- **Trigger**: `cargo run -- vm screenshot [output.png]`
- **QMP Command**: screendump to PPM
- **Conversion**: PPM → PNG via `magick` if available
- **Output**: Defaults to `tests/screenshots/vm_screenshot.png`
- **Fallback**: Saves PPM if magick unavailable

### VM Exec
- **Trigger**: `cargo run -- vm exec "command" --timeout 30`
- **Ephemeral**: Spawns fresh QEMU, not persistent session
- **Timeout**: Default 30 seconds per command
- **Output Processing**: Strips echo, prompts, ANSI codes
- **Shell Detection**: Waits for `# ` or `$ ` prompt

### VM Regs (Registers)
- **Trigger**: `cargo run -- vm regs [--qmp-socket path]`
- **Method**: QMP human-monitor-command `info registers`
- **Detection**: Auto-finds QMP socket or uses provided path
- **Output**: Formatted register dump

### VM Mem (Memory)
- **Trigger**: `cargo run -- vm mem 0xaddress [--len 256] [--qmp-socket path]`
- **Method**: QMP memsave command
- **Output**: Hex dump with ASCII sidebar
- **Format**: 16 bytes per line with offset labels

---

## Error Handling Behaviors

### Build Errors
- **Missing Dependencies**: Clear installation instructions printed
- **Build Failures**: Stops immediately, doesn't continue with dependent builds
- **File Not Found**: Specific error about missing file path

### Boot Errors
- **No Shell Prompt**: Timeout error after 30 seconds waiting
- **Kernel Panic**: Captured in serial output for analysis
- **QMP Socket**: Clear error if socket not created

### Test Errors
- **Golden File Mismatch**: Shows diff with context (Gold rating)
- **Auto-Update**: Silently updates (Silver rating)
- **Timeout**: Test fails after specified duration

### Run Errors
- **QEMU Not Found**: Fails preflight with installation instructions
- **Port Conflict**: websockify reports if port 6080 in use
- **VNC Setup**: Clear error if noVNC fails to clone

---

## File System Behaviors

### Directory Structure
```
toolchain/
  busybox-out/{arch}/busybox
  openrc-out/{arch}/
  openrc/{source}
  openrc-build/{build-artifacts}

target/
  initramfs/{arch}.cpio

tests/
  golden_boot_linux_openrc.txt
  screenshots/

.teams/
  TEAM_*.md

initramfs/
  initramfs.toml
```

### Artifact Cleanup
- **Command**: `cargo run -- clean`
- **Files Removed**:
  - `initramfs_*.cpio` (all arch variants)
  - `tinyos_disk.img`
  - `levitate.iso` (legacy)
  - `kernel64_rust.bin` (legacy custom kernel)
- **Directories Removed**:
  - `initrd_root`, `initrd_test_root` (legacy)
  - `iso_root` (legacy)
  - `limine-bin` (legacy)
- **Processes Killed**: Any remaining QEMU processes

### QMP Socket Management
- **Creation**: Only when explicitly enabled
- **Location**: `./qmp.sock` or `./qemu-session.sock`
- **Cleanup**: Auto-removed on QEMU exit or explicit cleanup
- **Validation**: Existence checked before connecting

### Serial Output
- **Default**: Printed to stdout for interactive modes
- **Test Mode**: Captured to analyze boot sequence
- **File Output**: Can redirect to `tests/behavior_{arch}.output`

---

## Configuration Behaviors

### Golden File Ratings
- **Gold**: Must match exactly, requires `--update` to refresh
- **Silver**: Auto-updates on every run, test always passes
- **Default**: Gold (safe for CI/CD)
- **Source**: `xtask.toml` golden_files section

### Architecture Selection
- **Default**: x86_64
- **Override**: `--arch aarch64` or `--arch x86_64`
- **Validation**: Fails if unsupported arch specified

---

## Performance Characteristics

### Build Times (Approximate)
- Linux kernel: 2-5 minutes (first time), 30 seconds (incremental)
- BusyBox: 30 seconds
- OpenRC: 1-2 minutes
- Initramfs: 5 seconds

### Boot Times
- Serial prompt: 5-10 seconds
- Full boot to shell: 10-15 seconds
- GUI mode: 15-20 seconds

### Memory Usage
- QEMU default: 2GB
- Pixel6 profile: 8GB
- Initramfs size: ~30-50 MB (compressed ~5-10 MB)

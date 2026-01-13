# TEAM_475: OpenRC Init System from Source

## Objective
Build OpenRC init system from source with musl static linking, enabling LevitateOS to have a real init system with service management, runlevels, and dependency tracking.

## Progress Log

### Session 1 (2026-01-13)
- Initial exploration of Alpine minirootfs approach
- Created download/extract functionality for Alpine rootfs
- User rejected approach: "no no no no... We're not just going to BE alpine..."

### Session 2 (2026-01-13) - OpenRC From Source
User requested building OpenRC from source instead of downloading Alpine packages.

**OpenRC Build System:**
- Created `xtask/src/build/openrc.rs` - Builds OpenRC 0.54 from source
- Uses meson build system with ninja backend
- Statically linked with musl-gcc
- Output: `toolchain/openrc-out/x86_64/`

**Build Challenges Solved:**
1. Static library vs shared library conflict - Used `--default-library=static`
2. Kernel headers for musl - Used `-idirafter /usr/include`
3. DESTDIR for install paths - Used `env("DESTDIR", &abs_install)`
4. Meson option types - `disabled` for features, `false` for booleans

**OpenRC Initramfs:**
- Created `create_openrc_initramfs()` function
- Combines BusyBox (shell + utilities) with OpenRC (init system)
- Includes all OpenRC binaries, shell scripts, and init.d services
- Creates proper runlevel directory structure

**Boot Testing:**
- Successfully boots Linux 6.19-rc5 with OpenRC init
- OpenRC 0.54 starts and processes runlevels
- Mounts /proc and /run correctly
- Runlevels execute: sysinit → boot → default

## Key Decisions

1. **Build from source**: User wanted to build their own OS, not just rebrand Alpine
2. **Static linking with musl**: Produces standalone binaries, no libc dependency
3. **BusyBox for utilities**: Keep BusyBox for shell and core utilities
4. **OpenRC for init**: Provides service management, runlevels, dependency tracking

## Gotchas Discovered

1. **Meson option types**: `audit` and `selinux` are "feature" type (use `disabled`), while `pam`, `bash-completions` etc. are "boolean" type (use `false`)

2. **Kernel headers with musl**: Adding `-I/usr/include` conflicts with glibc headers. Solution: Use `-idirafter /usr/include` to prioritize musl headers

3. **DESTDIR required**: OpenRC's meson.build uses absolute paths. Use `DESTDIR` env var to relocate install

4. **gendepends.sh relative paths**: OpenRC's gendepends.sh uses relative paths when scanning init.d scripts, causing warnings. These are non-fatal.

5. **rc-status in bin/**: Unlike other OpenRC binaries in sbin/, rc-status installs to bin/

6. **BusyBox tty setup with serial console**: When using `console=ttyS0` kernel parameter, the inittab entry `::wait:-/bin/ash` doesn't properly set up a controlling terminal. Use `getty -n -l /bin/ash 0 ttyS0 vt100` instead for auto-login with proper job control.

## Files Created/Modified

| File | Action |
|------|--------|
| `xtask/src/build/openrc.rs` | Created - OpenRC build from source |
| `xtask/src/build/mod.rs` | Modified - Export openrc and create_openrc_initramfs |
| `xtask/src/build/commands.rs` | Modified - Added Openrc, OpenrcInitramfs commands |
| `xtask/src/build/initramfs/mod.rs` | Modified - Added create_openrc_initramfs() |
| `xtask/src/main.rs` | Modified - Added --openrc flag, handlers |
| `xtask/src/run.rs` | Modified - Added run_qemu_term_linux(), openrc param |
| `xtask/src/qemu/builder.rs` | Modified - Added initrd() method |

## Usage

```bash
# Build OpenRC from source (one-time)
cargo xtask build openrc

# Build OpenRC initramfs
cargo xtask build openrc-initramfs

# Run with Linux kernel and OpenRC
cargo xtask run --linux --openrc --term
```

### Session 3 (2026-01-13) - BusyBox TTY Fix

Fixed the BusyBox-based initramfs (non-OpenRC) shell not being interactive.

**Problem**: Shell started but showed "can't access tty; job control turned off"

**Root Cause**:
- `::wait:-/bin/ash` in inittab doesn't properly set up controlling terminal
- With `console=ttyS0`, the serial device needs explicit handling

**Solution**:
1. Mount devtmpfs early so `/dev/ttyS0` exists
2. Use `getty -n -l /bin/ash 0 ttyS0 vt100` for auto-login with proper tty
3. Added terminal applets to manifest: getty, login, setsid, cttyhack, stty, tty

## Remaining Work

1. **Fix gendepends.sh warnings**: Init scripts need proper path handling
2. **Add more services**: hostname, localmount need to work
3. **Documentation**: Update CLAUDE.md with OpenRC usage

## Boot Output (Success)

```
OpenRC init version 0.54 starting
Starting sysinit runlevel
OpenRC 0.54 is starting up Linux 6.19.0-rc5-levitate (x86_64)
* Mounting /proc ... [ ok ]
* Mounting /run ... [ ok ]
* /run/openrc: creating directory
* Caching service dependencies ... [ ok ]
Starting boot runlevel
Starting default runlevel
```

## Handoff Notes

OpenRC 0.54 is successfully built from source and boots with the Linux kernel. The core functionality is working:
- OpenRC init starts
- Runlevels execute (sysinit, boot, default)
- /proc and /run mount correctly

Known issues:
- gendepends.sh warnings about relative paths (non-fatal)
- Some services don't start (missing dependencies or configs)

Next steps for future teams:
1. Fix init.d script path issues
2. Configure getty for console login
3. Add network service configuration

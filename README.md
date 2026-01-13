# LevitateOS

**An Experimental AI-Written Operating System**

> ⚠️ **This is NOT a production operating system.**
>
> LevitateOS is an experiment: *How far can I get building a general-purpose, POSIX-compatible, musl/BusyBox operating system written entirely by AI agents?*
>
> - Yes, the code looks bad — the goal is **making it work**, not winning beauty contests
> - No, I didn't care about security — only **capability**
> - Yes, a Linux app should run on it — that's the whole point
> - This is my own kernel, built from scratch in Rust
>
> **469+ AI team sessions** have contributed to this codebase. Each session is documented in `.teams/`.

**🔗 Kernel Repository:** [github.com/LevitateOS/kernel](https://github.com/LevitateOS/kernel) — The kernel is maintained as a standalone submodule for independent development.

---

## 🎯 The Goal

**Run unmodified Linux binaries.** Download a musl-linked binary, run it, it works.

```bash
# The dream:
$ apk add nginx
$ nginx
# It just works.
```

**Next milestone:** Run Alpine Linux's `apk` package manager. Since we share the same musl + BusyBox foundation as Alpine, their packages should just work.

---

## 📊 Current Status

| Component | Status | Notes |
|-----------|--------|-------|
| **BusyBox Shell (ash)** | ✅ Working | Interactive prompt, commands, pipes, job control |
| **fork/exec/wait** | ✅ Working | Full process lifecycle |
| **80+ coreutils** | ✅ Working | ls, cat, cp, mv, rm, mkdir, grep, sed, sort, etc. |
| **Command substitution** | ✅ Working | `$(echo hello)` works |
| **Pipes** | ✅ Working | `cat file | grep pattern` works |
| **File I/O** | ✅ Working | read, write, seek, stat, directories |
| **tmpfs** | ✅ Working | In-memory filesystem at /tmp |
| **devtmpfs** | ✅ Working | /dev/null, /dev/zero, /dev/urandom, /dev/full |
| **ext4** | 🟡 Read-only | Can read ext4 partitions, no write support yet |
| **FAT32** | ✅ Working | Boot partition support |
| **Signals** | 🟡 Partial | sigaction, sigprocmask work; delivery is basic |
| **procfs/sysfs** | ❌ Not yet | Blocking some programs that need /proc |
| **Networking** | ❌ Not yet | VirtIO-net driver exists, no TCP/IP stack |
| **Multi-user** | ❌ Not yet | Stubs only, runs as root |
| **Persistence** | ❌ Not yet | Requires ext4 write support |

---

## 🔥 What Actually Works Right Now

### Interactive Shell
```
LevitateOS# echo "Hello from BusyBox"
Hello from BusyBox

LevitateOS# ls /
bin   dev   etc   init  proc  root  sbin  sys   tmp

LevitateOS# cat /root/hello.txt
Hello from BusyBox initramfs!

LevitateOS# mkdir -p /tmp/test/nested && echo "created"
created

LevitateOS# echo "line1" > /tmp/file.txt && cat /tmp/file.txt
line1

LevitateOS# VAR=$(echo "substitution works") && echo $VAR
substitution works
```

## LevitateOS Syscall Implementation

### Overview
The dispatcher handles **124 syscalls** via a massive match statement.


### File System Syscalls (23)

| Category | Syscalls |
|----------|----------|
| **Basic I/O** | `read`, `write`, `open`¹, `close` |
| **Stat Operations** | `stat`¹, `fstat`, `lstat`¹, `fstatat`, `statx` |
| **Seek Operations** | `lseek`, `pread64`, `pwrite64` |
| **Directory Ops** | `openat`, `getdents`, `getdents64`² |
| **Working Dir** | `getcwd`, `chdir`, `fchdir` |
| **Permissions** | `chmod`, `fchmod`, `fchmodat` |
| **Ownership** | `chown`, `fchown`, `fchownat` |
| **Size Control** | `truncate`, `ftruncate` |
| **Access Check** | `access`¹, `faccessat` |

### Directory Syscalls (6)

- `mkdir`¹, `mkdirat`
- `unlink`¹, `unlinkat`
- `rmdir`¹
- `rename`¹, `renameat`

### Link Operations (5)

| Type | Syscalls |
|------|----------|
| **Symlinks** | `symlink`¹, `symlinkat`, `readlinkat` |
| **Hard Links** | `link` (via linkat), `linkat` |

### Time Syscalls (5)

- `gettimeofday` (legacy)
- `clock_gettime`, `clock_getres`
- `nanosleep`, `clock_nanosleep`

### Process Management (20)

| Category | Syscalls |
|----------|----------|
| **Identity** | `getpid`, `getppid`, `gettid` |
| **Spawn** | `spawn`³, `spawn_args`³ |
| **Exec/Exit** | `execve`, `exit`, `exit_group` |
| **Fork** | `clone`, `fork`, `vfork` |
| **Wait** | `waitpid`, `wait4` |
| **Process Groups** | `getpgid`, `getpgrp`², `setpgid`, `setsid` |
| **Thread Setup** | `set_tid_address` |
| **Terminal Control** | `set_foreground`³, `get_foreground`³ |

### User/Group Identity (12)

| Get | Set | Advanced |
|-----|-----|----------|
| `getuid` | `setuid` | `setreuid` |
| `geteuid` | `setgid` | `setregid` |
| `getgid` | | `setresuid`, `getresuid` |
| `getegid` | | `setresgid`, `getresgid` |

### Signal Handling (7)

- **Send**: `kill`, `tkill`
- **Wait**: `pause`, `rt_sigtimedwait`²
- **Management**: `sigaction`, `sigprocmask`, `sigreturn`, `sigaltstack`

### Memory Management (6)

| Core | Advanced |
|------|----------|
| `mmap`, `munmap`, `mprotect` | `madvise` |
| `brk`, `sbrk` | `pkey_alloc`⁴, `pkey_mprotect`⁴ |

### File Descriptor Operations (8)

- **Duplication**: `dup`, `dup2`, `dup3`
- **IPC**: `pipe2`
- **Control**: `fcntl`, `ioctl`
- **Utility**: `isatty`³

### Mount Operations (2)

- `mount`, `umount`

### Synchronization & I/O Multiplexing (8)

| Type | Syscalls |
|------|----------|
| **Futex** | `futex` |
| **Poll** | `poll`, `ppoll` |
| **Epoll** | `epoll_create1`, `epoll_ctl`, `epoll_wait` |
| **Events** | `eventfd2` |

### Socket Operations (4) - Mostly Stubbed

| Syscall | Status |
|---------|--------|
| `socket` | Stub → `EAFNOSUPPORT` |
| `socketpair` | Partial (pipe pair fallback) |
| `sendto` | Stub → `ENOTSOCK` |
| `sendfile` | Implemented |

### Vectored I/O (2)

- `readv`, `writev`

### Resource Management (3)

- `prlimit64`
- `getrusage`
- `getrandom`

### System Operations (4)

- `uname`
- `umask`
- `shutdown`, `reboot`
- `arch_prctl`² - TLS/GS register management

### Scheduler Operations (2)

- `sched_getaffinity`, `sched_setaffinity`

### Legend

- ¹ x86_64 legacy syscall
- ² x86_64 only
- ³ Custom LevitateOS syscall
- ⁴ Stub implementation

---

## 🏗️ Architecture

### Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| **QEMU x86_64** | ✅ Primary | q35 machine, daily development |
| **QEMU AArch64** | ✅ Working | virt machine, tested regularly |
| Intel NUC | 🔮 Aspirational | Future real hardware target |
| Pixel 6 | 🔮 Aspirational | Future ARM hardware target |

### Kernel Design

- **Higher-Half Kernel**: Runs at `0xFFFF_8000_0000_0000`
- **Preemptive Scheduler**: Round-robin with timer interrupts
- **Memory Management**: Buddy allocator (physical), VMA tracking (virtual)
- **VFS Layer**: Linux-inspired superblock → inode → dentry → file hierarchy
- **Linux ABI**: Implements Linux syscall numbers and struct layouts

### Userspace

- **libc**: musl (static linking)
- **Shell**: BusyBox ash
- **Utilities**: BusyBox provides 80+ commands
- **Init**: BusyBox init with `/etc/inittab`

### Directory Structure

```
crates/kernel/           # Kernel crates
├── levitate/            # Main kernel binary
├── arch/                # Architecture-specific (aarch64, x86_64)
├── syscall/             # Syscall implementations
├── sched/               # Scheduler and task management
├── mm/                  # Memory management
├── vfs/                 # Virtual filesystem
├── fs/                  # Filesystem implementations
│   ├── tmpfs/
│   ├── devtmpfs/
│   ├── initramfs/
│   ├── ext4/
│   └── fat/
├── drivers/             # Device drivers
│   ├── virtio-blk/
│   ├── virtio-gpu/
│   ├── virtio-input/
│   └── virtio-net/
└── hal/                 # Hardware abstraction layer

toolchain/               # Built externally
├── busybox/             # BusyBox source (cloned)
└── busybox-out/         # Compiled BusyBox binaries

xtask/                   # Build system
.teams/                  # AI team session logs (469+ sessions)
docs/                    # Documentation
```

---

## 🚀 Quick Start

### Prerequisites

```bash
# Rust nightly with components
rustup default nightly
rustup component add rust-src
rustup target add x86_64-unknown-linux-musl  # For userspace

# QEMU
sudo apt install qemu-system-x86 qemu-system-arm  # Ubuntu/Debian
sudo dnf install qemu-system-x86 qemu-system-arm  # Fedora

# For building BusyBox (optional, pre-built available)
sudo apt install musl-tools                        # Ubuntu/Debian
sudo dnf install musl-gcc musl-devel               # Fedora
```

### Build & Run

```bash
# Build and run (uses xtask under the hood)
./run.sh                     # GUI mode (default x86_64)
./run.sh --term              # Terminal mode (serial console)
./run.sh --vnc               # VNC display (browser at localhost:6080)
./run.sh --gdb               # Start GDB server
./run.sh clean               # Clean artifacts

# Alternative: direct xtask commands
cargo xtask build all
cargo xtask run
```

### Shell Scripts

| Script | Purpose |
|--------|--------|
| `./run.sh` | Main launcher (GUI, delegates to xtask) |
| `./run-term.sh` | Terminal mode - serial console in this terminal |
| `./run-vnc.sh` | VNC mode - view in browser at localhost:6080 |
| `./run-test.sh` | Run internal OS tests |

**Controls in terminal mode:**
- `Ctrl+A X` - Exit QEMU
- `Ctrl+A C` - Switch to QEMU monitor

### Testing

```bash
cargo xtask test             # All tests
cargo xtask test unit        # Host-side unit tests  
cargo xtask test behavior    # Golden log comparison
cargo xtask test regress     # Static analysis
```

---

## 📈 Development Progress

### Completed Milestones

1. **Kernel Boot** - Both architectures boot to userspace ✅
2. **Process Model** - fork/exec/wait working ✅
3. **BusyBox Integration** - Shell and 80+ utilities ✅
4. **Basic VFS** - tmpfs, devtmpfs, initramfs, FAT32, ext4 (read) ✅
5. **TTY/Job Control** - Interactive shell with proper terminal handling ✅
6. **musl libc** - Standard musl, same as Alpine Linux ✅

### Current Work

- **procfs/sysfs** - Needed for programs that read /proc (TEAM_469)
- **Coreutils Test Suite** - 83 tests passing, expanding coverage
- **Kernel Audit** - Proactive bug hunting (TEAM_468)

### Roadmap

| Epic | Status | Description |
|------|--------|-------------|
| **Process Model** | ✅ Done | fork, exec, wait, signals |
| **Filesystem Hierarchy** | 🟡 Partial | FHS structure, missing procfs/sysfs |
| **Users & Permissions** | ❌ Planned | Multi-user, chmod, chown, su |
| **Disk Persistence** | ❌ Blocked | Requires ext4 write support |
| **Networking** | ❌ Planned | TCP/IP stack needed |

---

## 🔧 Technical Details

### QEMU Configuration

| Architecture | Machine | RAM | CPU |
|--------------|---------|-----|-----|
| x86_64 | q35 | 1GB | qemu64 |
| AArch64 | virt | 1GB | cortex-a72 |

### Memory Layout

| Region | Physical | Virtual |
|--------|----------|---------|
| Device MMIO | `0x0..0x4000_0000` | Identity mapped |
| Kernel | `0x4008_0000` | `0xFFFF_8000_4008_0000` |
| User Stack | - | `0x7FFF_FFFF_0000` |
| User Heap | - | `0x0000_1000_0000` |

### Known Limitations

- **No dynamic linking** - All binaries must be statically linked
- **No networking** - VirtIO-net driver exists but no TCP/IP stack
- **Single user** - Runs everything as root, no permission enforcement
- **No persistence** - Changes lost on reboot (ext4 is read-only)
- **No /proc or /sys** - Some programs fail that need these

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [CLAUDE.md](CLAUDE.md) | AI agent guide (build commands, patterns) |
| [docs/GOTCHAS.md](docs/GOTCHAS.md) | 48+ known issues and how to avoid them |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Design principles |
| [docs/planning/MASTER_PLAN_GENERAL_PURPOSE_OS.md](docs/planning/MASTER_PLAN_GENERAL_PURPOSE_OS.md) | Roadmap to general-purpose OS |

### AI Team Logs

Every AI session is documented in `.teams/TEAM_XXX_*.md`. Recent highlights:

- **TEAM_459**: BusyBox ash shell fully working, 8 shell tests passing
- **TEAM_467**: Fixed uniq hang (fd 0/1/2 close bug)
- **TEAM_468**: Kernel audit - O_CLOEXEC, signal reset, 64-bit signals
- **TEAM_469**: Started procfs/sysfs planning

---

## 🤝 Contributing

This is an experimental project, but contributions are welcome!

- Read [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- Check `.teams/` for context on recent work
- See [docs/GOTCHAS.md](docs/GOTCHAS.md) before diving in

---

## 📄 License

LevitateOS is licensed under the **[MIT License](LICENSE)**.

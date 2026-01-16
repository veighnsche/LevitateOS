# LevitateOS Base System

## Target Stack (tentative)

| Layer | Choice | Notes |
|-------|--------|-------|
| Kernel | Linux | Standard |
| Init | systemd | Full-featured, widely supported |
| libc | musl | Lightweight, clean, static-friendly |
| Coreutils | Custom Rust | Minimal, musl-native (not GNU, not busybox) |

## Custom Coreutils (Rust)

Design goals:
- Written in Rust, targets musl
- NOT a busybox-style multi-call binary
- NOT GNU-compatible bloat (like uutils aims for)
- Just the essentials, done right

### What to include

**Essential (boot/init):**
- `cat`, `cp`, `mv`, `rm`, `mkdir`, `rmdir`
- `ls`, `ln`, `chmod`, `chown`
- `mount`, `umount`
- `echo`, `printf`, `test`, `[`
- `sleep`, `true`, `false`

**Shell support:**
- `head`, `tail`, `wc`, `cut`
- `grep` (or use ripgrep)
- `sort`, `uniq`, `tr`
- `env`, `printenv`

**System:**
- `id`, `whoami`, `groups`
- `uname`, `hostname`
- `date`, `touch`
- `df`, `du`
- `kill`, `ps` (maybe)

**Skip (use standalone tools):**
- `find` → use `fd`
- `grep` → use `ripgrep`
- `sed/awk` → use dedicated tools or script

### Build strategy

```bash
# Each tool is a separate binary
cargo build --release --target x86_64-unknown-linux-musl

# Or optionally: single binary with subcommands (like busybox but Rust)
levitate-core ls
levitate-core cp src dst
```

This is an unusual but valid combination. Most distros pair:
- glibc + GNU (Fedora, Debian, Arch)
- musl + busybox (Alpine)

LevitateOS: **musl + GNU** - best of both worlds.

## Why This Combination

- **systemd**: Modern init, service management, journald, networkd
- **musl**: Smaller, simpler, better static linking than glibc
- **GNU coreutils**: Full-featured `cp`, `ls`, `grep`, etc. (not stripped-down busybox)

## Challenges

- systemd assumes glibc (may need patches for musl)
- GNU coreutils builds fine with musl
- Some packages may need musl compatibility patches

## Alternatives Considered

| Option | Pros | Cons |
|--------|------|------|
| glibc + GNU | Maximum compatibility | Bloated libc |
| musl + busybox | Minimal | Missing features |
| musl + GNU | Clean libc + full tools | Less tested combo |

---

# Replacing glibc with musl

## Goal

LevitateOS uses Fedora kickstarts for installation, but replaces glibc with musl post-install using the local package manager.

## Why musl

- Smaller (~1MB vs ~10MB for glibc)
- Simpler, cleaner codebase
- Better static linking support
- No legacy bloat

## Approach

1. **Install**: Fedora kickstart bootstraps the system (glibc-based, temporary)
2. **First boot**: Package manager rebuilds everything from source with musl
3. **Result**: Pure musl system, glibc removed

## Package Manager Config

```lisp
(config
  (libc musl)
  (target "x86_64-unknown-linux-musl"))
```

All packages built from source inherit this config.

## Key Packages to Bootstrap

1. musl (the libc itself)
2. busybox or coreutils
3. init system
4. shell
5. package manager (self-hosting)

Once these are musl-linked, glibc can be removed.

## Static vs Dynamic

Prefer static linking during transition:
- Static musl binaries work regardless of installed libc
- Makes the switchover atomic

## Notes

- Kernel doesn't care about libc (kernel is self-contained)
- Some packages may need patches for musl compatibility
- Rust/Go binaries can easily target musl

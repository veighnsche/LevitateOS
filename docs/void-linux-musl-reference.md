# How Void Linux Does musl

Reference documentation for LevitateOS based on Void Linux's musl implementation.

---

## ⚠️ CURRENT vs FUTURE STATE

### Current LevitateOS Stack (glibc-based)
| Component | Current |
|-----------|---------|
| **libc** | glibc (dynamically linked) |
| **init** | systemd |
| **coreutils** | GNU coreutils |

### Future LevitateOS Stack (musl-based)
| Component | Future Goal |
|-----------|-------------|
| **libc** | musl (statically linked) |
| **init** | runit |
| **coreutils** | GNU coreutils (patched for musl, like Void does) |

**This document researches how Void Linux achieved the musl stack, as a roadmap for LevitateOS's future migration.**

---

## The Full Stack

### 1. Init System: runit (NOT systemd)

**Critical insight:** Void uses runit instead of systemd specifically because **systemd doesn't work with musl**.

> "runit is used for init(8) and service supervision. This allows Void to support musl as a second libc choice, which would not be possible with systemd."

runit is simple:
- Each service = a directory with a `run` script
- No complex dependencies
- Small codebase, easy to audit
- Works perfectly with musl

### 2. Build System: xbps-src

Void builds **separate binary packages** for glibc and musl from the **same source templates**.

```bash
# Build for glibc (default)
./xbps-src pkg firefox

# Build for musl (separate masterdir)
./xbps-src -A x86_64-musl pkg firefox
```

Key points:
- Uses dedicated masterdirs for each libc variant
- Chroot-based builds (xbps-uunshare, bwrap)
- Cross-compilation supported
- Templates can check `$XBPS_TARGET_LIBC` for libc-specific logic

### 3. Coreutils: GNU (Patched for musl)

**Void uses GNU coreutils on musl**, not busybox or an alternative. They patch it to work.

musl-specific configuration in Void's coreutils template:
- Disables `error_at_line` and `sys_cdefs.h` header checks for musl
- Sets `ac_cv_func_syncfs=no` for musl (syncfs() return value differences)
- Disables year2038 support for 32-bit musl (until musl 1.2.x)
- `kill` and `uptime` excluded (provided by util-linux and procps-ng)

Source: [void-packages/srcpkgs/coreutils/template](https://github.com/void-linux/void-packages/blob/master/srcpkgs/coreutils/template)

### 4. Supplementary Libraries

musl is strict about standards compliance and lacks some GNU extensions. Void created these packages:

| Package | Purpose |
|---------|---------|
| **musl-fts** | File tree traversal (`fts_open`, `fts_read`, etc.) - from NetBSD |
| **musl-obstack** | GNU obstack implementation - from gcc libiberty |
| **musl-legacy-compat** | Legacy headers (`sys/tree.h`, `sys/queue.h`, `sys/cdefs.h`) |

### 5. Patching Strategy

> "musl practices very strict standards compliance. Many commonly used platform-specific extensions are not present. Because of this, it is common for software to need modification."

Void's approach:
1. Patch incompatible software
2. Work with upstream to accept portability fixes
3. Use `archs="*-musl"` or `archs="~*-musl"` to control build targets
4. Check `$XBPS_TARGET_LIBC` in templates for conditional logic

---

## What Doesn't Work on musl

1. **NVIDIA proprietary drivers** - No musl support
2. **Most proprietary software** - Always glibc-linked
3. **V8-based software** - Historically broken on musl (QtWebEngine, etc.)
4. **32-bit/multilib** - No i686 musl support, no multilib repo
5. **Wine/32-bit games** - Requires multilib

### Workarounds

- **Flatpak** - Bundles glibc, works on musl host
- **glibc chroot** - Run incompatible software in a container
- **voidnsrun** - Tool to run glibc binaries on musl system

---

## How Void Made musl Work

### The Key Decisions

1. **runit over systemd** - Enabled musl support entirely
2. **Dual repositories** - Separate glibc and musl binary repos from same templates
3. **Supplementary libraries** - Fill gaps in musl (fts, obstack, legacy headers)
4. **Aggressive patching** - Fix software, upstream changes
5. **Clear documentation** - Users know limitations upfront

### Repository Structure

```
void-packages/
├── srcpkgs/
│   ├── musl/              # musl libc itself
│   ├── musl-fts/          # archs="*-musl"
│   ├── musl-obstack/      # archs="*-musl"
│   ├── musl-legacy-compat/
│   └── firefox/           # Same template, builds for both
└── hostdir/
    └── binpkgs/
        ├── x86_64/        # glibc packages
        └── x86_64-musl/   # musl packages
```

---

## Relevance to LevitateOS

### Current State
LevitateOS currently uses:
- **glibc** (dynamically linked)
- **systemd** for init
- **GNU coreutils**

### Future Migration Path
To migrate to a musl-based stack like Void, LevitateOS will need:

1. **Replace systemd with runit** - systemd does NOT work with musl
2. **Switch to musl libc** - Static linking preferred
3. **Patch GNU coreutils for musl** - Void's approach (or use uutils as alternative)
4. **Add supplementary libraries:**
   - musl-fts equivalent - If any packages need fts(3)
   - musl-obstack equivalent - If any packages use GNU obstack
5. **Patch incompatible packages** - Follow Void's patching strategy
6. **Document incompatibilities** - Users need to know what won't work

### What LevitateOS Already Has
- **S-expression recipes** - Similar concept to Void templates
- **uutils in vendor/** - Rust coreutils (alternative to patched GNU coreutils)

---

## Sources

- [Void Linux musl Handbook](https://docs.voidlinux.org/installation/musl.html)
- [void-packages GitHub](https://github.com/void-linux/void-packages)
- [void-packages Manual](https://github.com/void-linux/void-packages/blob/master/Manual.md)
- [musl-fts GitHub](https://github.com/void-linux/musl-fts)
- [musl-obstack GitHub](https://github.com/void-linux/musl-obstack)
- [void-runit GitHub](https://github.com/void-linux/void-runit)
- [xbps-src and musl announcement (2014)](https://voidlinux.org/news/2014/01/xbps-src-and-musl-libc.html)

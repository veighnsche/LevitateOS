# CLAUDE.md

## What is LevitateOS?

**A daily driver Linux distribution competing with Arch Linux.**

- Target: Power users who want control
- Philosophy: Base system, user builds up
- Package manager: `recipe` (Rhai-scripted, extracts from Fedora/Rocky RPMs)
- NOT embedded, NOT a container, NOT "minimal" - it's a complete desktop OS

---

## STOP. READ. THEN ACT.

**Read this before every action.**

- Before writing code → read what already exists
- Before deciding where code goes → check the Code Map below
- Before deleting anything → check `git status --ignored`

**Why:** On 2026-01-21, an agent created 500+ lines in the wrong location, then deleted it. Hours wasted, money burned, trust eroded.

---

## NEVER PIPE BUILD/TEST OUTPUT

**STRICTLY FORBIDDEN:**
```bash
# NEVER DO THIS:
cargo build 2>&1 | tail -20      # BLOCKS USER FROM SEEING ERRORS
cargo test 2>&1 | grep -i error  # USER WAITS 10 MINUTES FOR NOTHING
cargo run -- build 2>&1 | head   # HIDES REAL-TIME PROGRESS
```

**WHY:** When you pipe output, the user can't see what's happening. If a build fails at minute 3, they wait until the 10-minute timeout. They can't interrupt, they can't see the error, they can't help you.

**ALWAYS DO THIS:**
```bash
cargo build 2>&1              # Full streaming output
cargo test 2>&1               # User sees progress in real-time
cargo run -- build qcow2 2>&1 # User can ctrl+c if something goes wrong
```

The user monitors the output. When something breaks, they tell you. Don't hide it from them.

---

## Code Map: Where Things Live

```
LevitateOS/
├── leviso/                    # ISO BUILDER - builds LevitateOS ISO
│   ├── src/artifact/          #   Output artifacts (initramfs, ISO, squashfs)
│   ├── src/build/             #   Build phases (kernel, users, libdeps)
│   ├── src/component/         #   System components (firmware, packages, pam)
│   └── tests/                 #   Unit tests for leviso internals ONLY
│
├── distro-spec/               # SINGLE SOURCE OF TRUTH - specs & constants
│   └── src/
│       ├── shared/            #   Cross-distro (BootEntry, PartitionLayout, UserSpec)
│       ├── levitate/          #   LevitateOS-specific (systemd, paths)
│       └── acorn/             #   AcornOS-specific (OpenRC, Alpine)
│
├── tools/                     # USER-FACING TOOLS
│   ├── recipe/                #   Package manager (like pacman+AUR)
│   ├── recstrap/              #   System extractor (like pacstrap)
│   ├── recfstab/              #   Fstab generator (like genfstab)
│   ├── recchroot/             #   Chroot helper (like arch-chroot)
│   └── recqemu/               #   QEMU launcher (gui/vnc/serial modes)
│
├── testing/                   # ALL TEST CRATES
│   ├── install-tests/         #   E2E installation tests (QEMU)
│   ├── rootfs-tests/          #   User experience tests (nspawn)
│   ├── hardware-compat/       #   Hardware compatibility
│   ├── cheat-guard/           #   Anti-cheat macros (runtime)
│   └── cheat-test/            #   Anti-cheat macros (proc-macro)
│
├── leviso-elf/                # ELF analysis & library copying
├── distro-builder/            # Shared ISO building infrastructure
├── AcornOS/                   # [SUBMODULE] Alpine-based variant
├── linux/                     # [SUBMODULE] Linux kernel source
├── vendor/                    # Reference implementations
├── docs/                      # Documentation (content + TUI viewer)
├── llm-toolkit/               # LoRA training toolkit
└── .teams/                    # Work history & knowledge base
```

### Where to Implement Things

| If you need to... | Go to | NOT to |
|-------------------|-------|--------|
| Define boot/partition/user specs | `distro-spec/src/shared/` | leviso |
| Add LevitateOS constants or paths | `distro-spec/src/levitate/` | leviso |
| Build ISO, initramfs, squashfs | `leviso/src/artifact/` | — |
| Add system components | `leviso/src/component/` | — |
| Write E2E installation tests | `testing/install-tests/` | `leviso/tests/` |
| Write rootfs/UX tests | `testing/rootfs-tests/` | leviso |
| Add user-facing installer tools | `tools/` | leviso |
| Add package manager features | `tools/recipe/` | — |
| Add ELF/library utilities | `leviso-elf/` | leviso |
| Add auth/login specifications | `distro-spec/src/shared/auth/` | leviso |

### Authentication Subsystem

**Location**: `distro-spec/src/shared/auth/`

All authentication and login configuration is consolidated in the auth subsystem:

- `mod.rs` - Public API and architecture overview
- `requirements.md` - Complete 700+ line specification document
- `components.rs` - Auth binaries, PAM modules, security configs
- `pam.rs` - 12+ PAM configuration file contents
- `getty.rs` - Console/serial login constants
- `ssh.rs` - SSH server configuration constants

**What goes here**:
- PAM configuration file contents (pam.rs)
- Component lists (AUTH_BIN, AUTH_SBIN, PAM_MODULES, etc.)
- Security file paths (SECURITY_FILES)
- Configuration constants (getty, ssh)
- Specifications and requirements documentation

**What doesn't go here**:
- PAM file creation logic (stays in `leviso/src/component/custom/pam.rs`)
- Live overlay creation (stays in `leviso/src/component/custom/live.rs`)
- Build orchestration

**Key Insight**: distro-spec = data/specifications, leviso = build logic. Don't mix them.

**Critical Files** (if something fails to login):
- Must verify: `/usr/bin/login` symlink (agetty searches PATH)
- Must verify: `/usr/sbin/unix_chkpwd` (pam_unix.so hardcoded path)
- Must verify: PAM configs are correct (12 files, specific auth stacks)
- Must verify: `/etc/shadow` has 0600 permissions (secret)

See `distro-spec/src/shared/auth/requirements.md` for full specifications.

### Common Mistakes

| Wrong | Right |
|-------|-------|
| `leviso/tests/install_test.rs` | `testing/install-tests/` |
| Hardcode boot entries in leviso | `distro_spec::levitate::default_boot_entry()` |
| Duplicate utility functions | Check `leviso/src/common/` first |
| Add constants in multiple places | Add to `distro-spec`, import elsewhere |

---

## Global Rules

### 1. Required features = ON by default
UEFI is required → no `--uefi` flag, use `--bios` to opt out.

### 2. Ask "What does archiso do?"
Live ISO should match archiso: autologin, root shell, manual install via `recstrap`.

### 3. Ask before architecture decisions
Don't silently add workarounds.

### 4. Question "is this necessary?" BEFORE building
TEAM_075 wasted tokens building a bootstrap system before realizing it wasn't needed.

### 5. Check vendor/ before inventing solutions
```bash
grep -rn "your_problem" vendor/systemd/
```

### 6. No false positives in tests
Missing item = test fails. Never move to "optional" just to pass.

### 7. FAIL FAST
Required component missing? `bail!()`, not `println!("Warning...")`.

### 8. Before deleting directories
```bash
git status --ignored  # Gitignored ≠ worthless
```

### 9. Never say "minimal"
If something's missing that a daily-driver desktop needs, that's a BUG.

### 10. LLM is a tool, not the identity
Marketing: "You write recipes. A local LLM can help." NOT "AI writes packages for you."

---

## Website Design System (`docs/website/`)

**Target viewport:** 960×1080 — half of a 1080p screen. Power users tile windows (docs on left, terminal on right). Design for split-screen, not full-screen browsing.

### Design Rules

| Rule | Spec |
|------|------|
| Rounded corners | **NONE.** `global.css` enforces `* { border-radius: 0 !important; }` |
| Body text | `text-sm` (14px) — readable at 960px width |
| Page titles | `text-2xl font-medium` |
| Section headings | `text-lg font-medium` (h2) or `text-base font-medium` (h3) |
| Card titles | `text-base font-medium` or `text-lg font-medium` |
| Primary buttons | `h-10 px-5 text-sm` |
| Secondary buttons | `h-10 px-5 text-sm` |
| Section padding | `py-16` for main sections, `py-6` for smaller sections |
| Card padding | `p-5` or `p-6` |
| Gaps | `gap-2`, `gap-3`, `gap-4`, `gap-6` — comfortable spacing |
| SVG icons | `size-5` for buttons/headers, `size-4` for inline |
| Interactive elements | Add `select-none` |
| Font weight | `font-medium`, never `font-bold` or `font-semibold` |

### What NOT to Do

```tsx
// WRONG - rounded corners (CSS reset catches this, but don't add them)
<button class="rounded-lg">

// WRONG - too small for 960px viewport
<p class="text-xs">Hard to read body text</p>

// WRONG - oversized marketing-style padding
<section class="py-32 px-20">

// RIGHT
<button class="h-10 px-5 text-sm font-medium select-none">
<p class="text-sm text-muted-foreground">Readable body text</p>
<section class="py-16">
```

### Why This Matters

The CSS reset (`* { border-radius: 0 !important; }`) enforces no rounded corners site-wide. This cannot be bypassed.

For font sizes: design for legibility at 960×1080. `text-sm` (14px) is the baseline for body text. Anything smaller becomes hard to read in a split-screen workflow.

---

## Target Hardware Profile

When reasoning about hardware, think **modern desktop/laptop** not server/embedded:

| Component | Minimum | Typical User | Power User |
|-----------|---------|--------------|------------|
| Display | 1920x1080 | 2560x1440 | 3840x2160 |
| RAM | 8 GB | 16-32 GB | 64-128 GB |
| Storage | 64 GB NVMe | 256-512 GB NVMe | 1-4 TB NVMe |
| CPU | x86-64-v3 | Ryzen 5 / i5 | Ryzen 9 / i9 |
| GPU | Integrated | RTX 3060 / RX 6700 | RTX 4090 / RX 7900 |

**WRONG mental model:** "What's the minimum to boot?"
**RIGHT mental model:** "What would a developer or gamer have?"

**NEVER use 1024x768 or other legacy resolutions for testing.** This is 2026, not 1999.

Reference: `distro_spec::shared::LEVITATE_REQUIREMENTS`

---

## Commands

```bash
cd leviso
cargo run -- build            # Full build
cargo run -- build initramfs  # Rebuild initramfs only
cargo run -- build iso        # Rebuild ISO only
cargo run -- run              # Boot in QEMU
```

---

## Visual Testing with Puppeteer + noVNC

When you need to visually test the ISO (type commands, take screenshots):

**RESOLUTION: 1920x1080 - ALWAYS**

This is a daily-driver desktop OS, not a toy. Use full HD resolution for all visual testing.
Never use 1024x768 or other legacy resolutions.

**1. Start QEMU + websockify** (in background):
```bash
recqemu vnc leviso/output/levitateos.iso --websockify &
```

**2. Connect + Type + Screenshot** (MCP tool calls):
```
mcp__puppeteer__puppeteer_navigate  url="http://localhost:6080/vnc.html?autoconnect=true"
# wait 3 seconds for connection
mcp__puppeteer__puppeteer_screenshot  name="boot" width=1920 height=1080
mcp__puppeteer__puppeteer_fill  selector="#noVNC_keyboardinput" value="echo hello\n"
mcp__puppeteer__puppeteer_screenshot  name="after-echo" width=1920 height=1080
```

Key: Always use `#noVNC_keyboardinput` for typing, always add `\n` for Enter.

Full docs: `.teams/KNOWLEDGE_visual-install-testing.md`

---

## First Step: Create Team File

```bash
ls .teams/TEAM_*.md | tail -1  # Find next number
# Create: .teams/TEAM_XXX_short-description.md
```

Do this BEFORE any code changes.

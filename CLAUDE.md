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
├── tools/                     # USER-FACING INSTALLER TOOLS
│   ├── recipe/                #   Package manager (like pacman+AUR)
│   ├── recstrap/              #   System extractor (like pacstrap)
│   ├── recfstab/              #   Fstab generator (like genfstab)
│   └── recchroot/             #   Chroot helper (like arch-chroot)
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

## Target Hardware Profile

When reasoning about hardware, think **modern desktop/laptop** not server/embedded:

| Component | Minimum | Typical User | Power User |
|-----------|---------|--------------|------------|
| RAM | 8 GB | 16-32 GB | 64-128 GB |
| Storage | 64 GB NVMe | 256-512 GB NVMe | 1-4 TB NVMe |
| CPU | x86-64-v3 | Ryzen 5 / i5 | Ryzen 9 / i9 |
| GPU | Integrated | RTX 3060 / RX 6700 | RTX 4090 / RX 7900 |

**WRONG mental model:** "What's the minimum to boot?"
**RIGHT mental model:** "What would a developer or gamer have?"

Reference: `distro_spec::shared::LEVITATE_REQUIREMENTS`

---

## Commands

```bash
cd leviso
cargo run -- build      # Full build
cargo run -- initramfs  # Rebuild initramfs
cargo run -- iso        # Rebuild ISO
cargo run -- run        # Boot in QEMU
```

---

## Visual Testing with Puppeteer + noVNC

When you need to visually test the ISO (type commands, take screenshots):

**1. Start QEMU + websockify** (in background):
```bash
qemu-system-x86_64 -enable-kvm -m 4G -cpu host \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/ovmf/OVMF_CODE.fd \
  -cdrom output/levitateos.iso -vnc :0 -device virtio-vga -boot d -display none &
websockify 6080 localhost:5900 --web /tmp/novnc &
```

**2. Connect**: `puppeteer_navigate` → `http://localhost:6080/vnc.html?autoconnect=true`

**3. Type commands**: `puppeteer_fill` selector=`#noVNC_keyboardinput` value=`"command\n"`

**4. Screenshot**: `puppeteer_screenshot` name=`"name"` width=`1024` height=`768`

Full docs: `.teams/TEAM_127_visual-install-testing.md`

---

## First Step: Create Team File

```bash
ls .teams/TEAM_*.md | tail -1  # Find next number
# Create: .teams/TEAM_XXX_short-description.md
```

Do this BEFORE any code changes.

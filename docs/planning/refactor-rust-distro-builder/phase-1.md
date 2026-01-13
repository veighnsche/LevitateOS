# Phase 1: Safeguards

## Objective

Establish safety nets before making any destructive changes.

---

## Tasks

### 1.1 Tag Current State

```bash
git tag v1.0-custom-kernel -m "Last version with custom kernel"
```

This preserves the exact commit for rollback if needed.

### 1.2 Create Archive Branch

```bash
git checkout -b archive/custom-kernel
git checkout main
```

The `archive/custom-kernel` branch preserves the full custom kernel code with history.

### 1.3 Verify Current Behavior Works

Before changing anything, confirm the current default path works:

```bash
cargo xtask run --term
# Expected: Boots to OpenRC shell prompt
# Ctrl+A X to exit
```

### 1.4 Capture Golden Output

Create a golden file for the Linux + OpenRC boot:

```bash
timeout 30 cargo xtask run --term 2>&1 | head -100 > tests/golden_boot_linux_openrc.txt
```

Edit to keep key lines:
- Linux version
- OpenRC startup messages
- Shell prompt

### 1.5 Document What Exists

Inventory of current state:

| Directory | LOC | Action |
|-----------|-----|--------|
| `crates/kernel/` | ~30,000 | Archive, then delete |
| `crates/userspace/` | ~11,000 | Archive, then delete |
| `xtask/src/` | ~5,000 | Keep (this is the product) |
| `initramfs/` | ~200 | Keep, move to `config/` |
| `.external-kernels/` | N/A | Delete |
| `scripts/` | ~100 | Delete |
| `qemu/` | ~50 | Delete |

---

## Verification

Before proceeding to Phase 2:

- [ ] Tag `v1.0-custom-kernel` exists
- [ ] Branch `archive/custom-kernel` exists
- [ ] `cargo xtask run --term` boots to shell
- [ ] Golden file `tests/golden_boot_linux_openrc.txt` created
- [ ] This checklist documented in team file

---

## Rollback Plan

If anything goes wrong in later phases:

```bash
# Return to pre-refactor state
git checkout v1.0-custom-kernel

# Or restore specific files
git checkout archive/custom-kernel -- path/to/file
```

---

## Checklist

- [ ] `git tag v1.0-custom-kernel`
- [ ] `git checkout -b archive/custom-kernel`
- [ ] Verified `cargo xtask run --term` works
- [ ] Created golden file
- [ ] Ready for Phase 2

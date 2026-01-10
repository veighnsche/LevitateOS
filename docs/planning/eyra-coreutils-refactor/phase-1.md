# Phase 1: Discovery and Cleanup

## Objective
Understand the current mess and remove stale artifacts before restructuring.

## Current State Analysis

### Folder Structure (BROKEN)
```
crates/userspace/eyra/
├── .cargo/config.toml      ← Workspace config (CORRECT)
├── Cargo.toml              ← Workspace manifest (CORRECT)
├── Cargo.lock              ← Workspace lock (CORRECT)
├── target/                 ← Workspace target (CORRECT)
├── rust-toolchain.toml     ← Pinned toolchain (CORRECT)
├── cat/
│   ├── .cargo/             ← STALE (duplicate)
│   ├── Cargo.lock          ← STALE (duplicate)
│   ├── target/             ← STALE (802MB waste)
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
├── cp/
│   ├── .cargo/             ← STALE
│   ├── Cargo.lock          ← STALE
│   ├── target/             ← STALE (329MB)
│   ...
└── (13 more utilities with same duplication)
```

### What Needs Removal
- 15× `target/` folders (5.1GB total)
- 15× `Cargo.lock` files (workspace lock is authoritative)
- 15× `.cargo/` folders (workspace config is authoritative)

## Success Criteria
- [ ] All per-utility `target/` folders deleted
- [ ] All per-utility `Cargo.lock` files deleted  
- [ ] All per-utility `.cargo/` folders deleted
- [ ] Workspace `target/` still works
- [ ] `./run-test.sh` still passes

## Steps

### Step 1: Verify Current Tests Pass
Before any cleanup, ensure baseline works:
```bash
./run-test.sh
```
Record output for comparison after cleanup.

### Step 2: Remove Stale Artifacts
```bash
# Remove stale target folders (5.1GB)
find crates/userspace/eyra -mindepth 2 -name "target" -type d -exec rm -rf {} +

# Remove per-utility Cargo.lock (workspace has authoritative lock)
find crates/userspace/eyra -mindepth 2 -name "Cargo.lock" -delete

# Remove per-utility .cargo folders (workspace has authoritative config)
find crates/userspace/eyra -mindepth 2 -name ".cargo" -type d -exec rm -rf {} +
```

### Step 3: Verify Build Still Works
Run `./run-test.sh` and confirm all tests pass.

### Step 4: Handoff Verification
- [ ] `./run-test.sh` output identical to Step 1
- [ ] `ls crates/userspace/eyra/*/target` shows no results
- [ ] `ls crates/userspace/eyra/*/Cargo.lock` shows no results

## Risks
- LOW: Removing stale folders shouldn't affect workspace builds
- Workspace already uses `eyra/target/` for all builds

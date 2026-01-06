# Phase 4: Integration and Testing — Levbox Completion

**Phase:** Integration and Testing  
**Status:** In Progress  
**Team:** TEAM_209

---

## 1. Unit of Work Breakdown

### 1.1 Step 1: Initramfs Verification
- [ ] Rebuild all userspace utilities.
- [ ] Run `scripts/make_initramfs.sh` and verify output.

### 1.2 Step 2: Runtime Verification (QEMU)
- [ ] Boot LevitateOS.
- [ ] Test `touch /tmp/test_file`.
- [ ] Test `ls -l /tmp` to verify timestamps.
- [ ] Test `ln -s /tmp/test_file /tmp/test_symlink`.
- [ ] Test `ln /tmp/test_file /tmp/test_hardlink`.
- [ ] Verify `ls -l /tmp` shows link counts and correct types.
- [ ] Test `rm` on links and original file.

---

## 2. Unit of Work Details

### 1.1.1 UoW 1: Userspace Build & Initramfs
- **Goal:** Ensure all 10 utilities are packaged.
- **Tasks:**
  - `cargo build --release -p levbox`
  - `./scripts/make_initramfs.sh`

### 1.2.1 UoW 2: Manual Testing in QEMU
- **Goal:** Confirm syscalls work as expected.
- **Tasks:**
  - `cargo xtask run`
  - Run the test sequence in the shell.

# Phase 5 — Hardening and Handoff

**TEAM_311**: ABI Stability Refactor
**Parent**: `docs/planning/stability-maturation/`
**Depends On**: Phase 4 complete
**Status**: Pending

---

## 1. Final Verification

### 1.1 Build Verification
- [ ] `cargo build --workspace` passes
- [ ] `cargo build --target aarch64-unknown-none` passes
- [ ] `cargo build --target x86_64-unknown-none` passes

### 1.2 Test Verification
- [ ] `cargo test -p los_abi --features std` passes
- [ ] `cargo test -p los_utils --features std` passes
- [ ] Golden boot tests pass (aarch64)
- [ ] Golden boot tests pass (x86_64)

### 1.3 Runtime Verification
- [ ] Boot to shell (aarch64)
- [ ] Boot to shell (x86_64)
- [ ] Process creation works (fork+exec pattern)
- [ ] Shell commands execute correctly

---

## 2. Documentation Updates

### 2.1 Files to Update
| File | Update |
|------|--------|
| `docs/ARCHITECTURE.md` | Add los_abi to crate list |
| `docs/specs/userspace-abi.md` | Remove custom syscalls section |
| `README.md` | Update crate structure if mentioned |

### 2.2 New Documentation
- [ ] `crates/abi/README.md` - Crate purpose and usage
- [ ] Update ADDING_NEW_ARCHITECTURE.md if affected

---

## 3. Regression Test Suite

### 3.1 New Tests Added
| Test | Location | Verifies |
|------|----------|----------|
| Size assertions | `crates/abi/src/lib.rs` | ABI struct sizes match Linux |
| Errno consistency | `crates/abi/src/errno.rs` | Error codes match Linux |
| Syscall numbers | `crates/abi/src/syscall/mod.rs` | Numbers match linux-raw-sys |

### 3.2 Golden Test Updates
If boot output changed:
- [ ] Verify changes are intentional
- [ ] Update `tests/golden_boot.txt`
- [ ] Update `tests/golden_boot_x86_64.txt`

---

## 4. Handoff Checklist

### 4.1 Code Quality
- [ ] No `TODO(TEAM_311)` comments remaining
- [ ] No `#[allow(dead_code)]` on new code
- [ ] All public items documented
- [ ] Clippy clean

### 4.2 Team File Update
Update `.teams/TEAM_311_stability_audit_abi_api.md`:
- [ ] Status: Complete
- [ ] Summary of changes
- [ ] Files added/removed
- [ ] Breaking changes documented

### 4.3 Future Work Notes
Document any deferred items:
- HAL `static mut` cleanup (separate refactor)
- Boot phase type-state pattern (separate refactor)
- IRQ handler standardization (separate refactor)

---

## 5. Steps

### Step 1 — Run Full Verification Suite
- [ ] All builds pass
- [ ] All tests pass
- [ ] Runtime smoke test

### Step 2 — Update Documentation
- [ ] Architecture docs
- [ ] ABI spec
- [ ] Crate README

### Step 3 — Final Team File Update
- [ ] Mark status complete
- [ ] Document all changes
- [ ] Note any follow-up work

### Step 4 — Handoff
- [ ] Commit all changes
- [ ] Tag release if appropriate
- [ ] Close planning phase

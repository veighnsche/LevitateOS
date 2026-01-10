# Phase 3: Fix Design and Validation Plan

**TEAM_365** | Eyra/uutils Linker Conflict  
**Created:** 2026-01-10

---

## 1. Root Cause Summary

**What's wrong:** Some uutils crates (via uucore's `i18n-common`/`icu_locale` features) bring in ICU dependencies that define C runtime symbols (`_start`, `__dso_handle`) conflicting with Eyra's pure-Rust entry point.

**Where it lives:** Transitive dependency chain:
```
uu_echo → uucore[format] → uucore[i18n-common] → uucore[icu_locale] → icu_* crates
```

---

## 2. Fix Strategy Options

### Option A: Disable i18n Features (RECOMMENDED)
Disable `default-features` and specify only needed features without i18n.

**Pros:**
- Simple, no patches required
- Smaller binaries
- Works immediately

**Cons:**
- May lose some functionality (localized error messages)
- Need to identify minimal feature set per utility

**Reversibility:** Easy — just remove `default-features = false`

---

### Option B: Linker Flag Workaround
Add `--allow-multiple-definition` to linker flags.

**Pros:**
- Quick fix
- Keeps all features

**Cons:**
- May cause runtime issues if wrong symbol is used
- Masks the real problem
- Not a clean solution

**Reversibility:** Easy — remove linker flag

---

### Option C: Fork uutils
Fork and patch uutils to avoid ICU.

**Pros:**
- Full control

**Cons:**
- Maintenance burden
- Diverges from upstream

**Reversibility:** Medium — need to maintain fork

---

### Option D: Report Upstream
Report to Eyra and/or uutils projects.

**Pros:**
- Proper fix for everyone

**Cons:**
- May take time
- Not immediate solution

**Reversibility:** N/A

---

## 3. Chosen Fix: Option A

**Approach:** For each blocked utility, configure with `default-features = false` and add only required features.

### 3.1 Feature Requirements Research

| Utility | Required Features | Notes |
|---------|------------------|-------|
| echo | (none) | Basic echo needs no features |
| env | (none) | Basic env needs no features |
| true | (none) | No features needed |
| false | (none) | No features needed |
| touch | fs | File operations |
| rm | fs | File operations |
| rmdir | fs | File operations |
| ln | fs | File operations |
| cp | fs | File operations |
| mv | fs | File operations |

---

## 4. Reversal Strategy

If fix doesn't work:
1. Revert Cargo.toml changes
2. Fall back to hand-written implementations
3. Document which utilities cannot use uutils

**Signals to revert:**
- Build still fails after feature changes
- Runtime errors due to missing features
- Core functionality broken

---

## 5. Test Strategy

### 5.1 Build Tests
For each fixed utility:
```bash
cargo build --release --target x86_64-unknown-linux-gnu -Zbuild-std=std,panic_abort
```

### 5.2 Functional Tests
Run in LevitateOS:
- `echo "hello"` → outputs "hello"
- `env` → lists environment variables
- `touch /tmp/test` → creates file
- etc.

### 5.3 Regression Protection
Add CI check that all Eyra utilities build successfully.

---

## 6. Impact Analysis

### API Changes
None — same command-line interface

### Behavior Changes
- No localized error messages (English only)
- Otherwise identical

### Performance
- Smaller binaries (no ICU)
- Faster startup (less initialization)

---

## 7. Implementation Steps

### Step 1: Fix echo
```toml
[dependencies]
uu_echo = { version = "0.2", default-features = false }
```

### Step 2: Build and verify

### Step 3: If successful, apply pattern to remaining blocked utilities

### Step 4: Clean up failed utility directories (coreutils-true, coreutils-false)

### Step 5: Update team documentation

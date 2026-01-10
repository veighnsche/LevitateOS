# Phase 2: Root Cause Analysis

**TEAM_365** | Eyra/uutils Linker Conflict  
**Created:** 2026-01-10

---

## 1. Key Finding: Feature Differences

### Working (uu_cat)
```
uu_cat → uucore features: default, fast-inc, fs, pipes
Dependencies: dunce, libc (simple)
```

### Blocked (uu_echo)
```
uu_echo → uucore features: default, format, bigdecimal, extendedbigdecimal, 
                           num-traits, itertools, parser, glob, quoting-style,
                           i18n-common, icu_locale
Dependencies: ICU, bigdecimal, displaydoc, num-* (complex)
```

**Pattern:** Blocked utilities enable `i18n-common` and/or `icu_locale` features.

---

## 2. Hypotheses

### H1: ICU crates define conflicting symbols — **DISPROVEN**
**TEAM_366 tested this:** Disabling ICU via `default-features = false` did not fix the issue.
The `_start` symbol comes from the binary's own object file, not ICU.

Original hypothesis (HIGH confidence → **WRONG**):
The ICU (International Components for Unicode) crates may define `_start` or `__dso_handle` for their own runtime initialization.

**Evidence:**
- Only utilities with `icu_locale` feature fail
- ICU is a large C/C++ library with Rust bindings
- C runtime symbols like `__dso_handle` are typical in C interop

### H2: uucore `format` feature brings C dependencies (MEDIUM confidence)
The format feature enables parsing/formatting that may use C libraries.

**Evidence:**
- uu_echo uses `format` feature
- Format feature enables parser, quoting-style, i18n

### H3: Build-std interaction with C libraries (MEDIUM confidence)
`-Zbuild-std` may not properly handle crates that expect system libc.

**Evidence:**
- Eyra provides its own libc via c-scape
- Some dependencies may still link against system crt objects

---

## 3. Investigation Strategy

### Step 1: Verify H1 - Check if disabling icu_locale fixes the issue
Try building uu_echo with `default-features = false` and minimal features.

### Step 2: Identify exact symbol source
Use `nm` or `objdump` to find which object file defines `_start`.

### Step 3: Check other blocked utilities' features
Map which blocked utilities have icu_locale vs which don't.

---

## 4. Step 1 Results: Feature Analysis

| Utility | Status | Has icu_locale? | Has format? |
|---------|--------|-----------------|-------------|
| cat | ✅ | No | No |
| pwd | ✅ | No | No |
| mkdir | ✅ | No | No |
| ls | ✅ | ? | ? |
| echo | ❌ | Yes | Yes |
| true | ❌ | ? | ? |
| false | ❌ | ? | ? |
| env | ❌ | ? | ? |

Need to verify remaining utilities.

---

## 5. Potential Fixes

### Fix A: Disable problematic features
```toml
uu_echo = { version = "0.2", default-features = false, features = ["..."] }
```

### Fix B: Use linker flags to ignore duplicates
```toml
[build]
rustflags = ["-C", "link-args=-Wl,--allow-multiple-definition"]
```

### Fix C: Patch uutils to not pull ICU
Fork or configure uutils to avoid i18n features.

### Fix D: Report upstream to Eyra
May be a known compatibility issue with certain crate patterns.

---

## 6. Next Steps

1. Test Fix A on uu_echo
2. If successful, apply to other blocked utilities
3. Document which features are safe to disable
4. Create regression test

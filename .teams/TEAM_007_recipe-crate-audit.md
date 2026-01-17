# TEAM_007: Recipe Crate Audit

## Task
Proactively find bugs, gaps, and improvement opportunities in the recipe crate.

## Status
- [x] Audit parser.rs for parsing bugs
- [x] Audit recipe.rs for data structure issues
- [x] Audit executor.rs for execution bugs
- [x] Audit levitate.rs CLI for usability issues
- [x] Review recipe files for consistency
- [x] Document findings and recommendations

---

## CRITICAL BUGS FOUND

### BUG #1: Dependency Parsing Completely Broken (SEVERITY: CRITICAL)

**Location:** `recipe.rs:245-258`

**Problem:** The recipe files use a nested format for dependencies:
```lisp
(deps
  (build "meson" "ninja" "pkg-config")
  (runtime "wlroots" "wayland"))
```

But the parser expects a flat format:
```lisp
(deps meson ninja pkg-config)
(build-deps meson ninja pkg-config)
```

The parser uses `filter_map(|e| e.as_atom())` which silently discards lists, so the nested `(build ...)` and `(runtime ...)` expressions are completely ignored.

**Impact:** ALL 26 recipes have broken dependency parsing. The `levitate desktop` command will NOT install dependencies correctly.

**Test to reproduce:**
```rust
let input = r#"(package "sway" "1.10"
  (deps (build "meson") (runtime "wayland")))"#;
let recipe = Recipe::from_expr(&parse(input).unwrap()).unwrap();
assert!(recipe.deps.is_empty());      // BUG: returns []
assert!(recipe.build_deps.is_empty()); // BUG: returns []
```

---

### BUG #2: SHA256 Verification Not Parsed (SEVERITY: HIGH)

**Location:** `recipe.rs:306-313`

**Problem:** The `parse_acquire` function for source URLs ignores the nested sha256 verification:
```lisp
(acquire
  (source "https://example.com/file.tar.gz"
    (sha256 "abc123...")))
```

The parser only reads the URL and sets `verify: None`.

**Impact:** Checksum verification is silently skipped, making downloads insecure.

---

### BUG #3: Git References Not Parsed (SEVERITY: MEDIUM)

**Location:** `recipe.rs:329-335`

**Problem:** Git reference (tag/branch/commit) is not parsed:
```lisp
(acquire
  (git "https://github.com/test/repo.git" (tag "v1.0")))
```

The parser only reads the URL and sets `reference: None`.

**Impact:** Git clones will always use the default branch instead of the specified tag/branch/commit.

---

## GAPS / INCOMPLETE IMPLEMENTATIONS

### GAP #1: Configure Phase Not Implemented

**Location:** `recipe.rs:456-458`

```rust
fn parse_configure(_expr: &Expr) -> Result<Option<ConfigureSpec>, RecipeError> {
    // TODO: implement
    Ok(None)
}
```

**Impact:** Post-install configuration (creating users, directories, templates) cannot be specified in recipes.

---

### GAP #2: Sandbox Execution Not Implemented

**Location:** `executor.rs:594-597`

```rust
StartSpec::Sandbox { config: _, exec } => {
    // TODO: implement sandboxing with landlock/seccomp
    exec.join(" ")
}
```

**Impact:** Sandbox security feature is non-functional.

---

### GAP #3: Update/Hooks Actions Not Implemented

**Location:** `recipe.rs:283-285`

```rust
"update" | "hooks" => {
    // TODO: implement these
}
```

**Impact:** Package update mechanisms and lifecycle hooks are not available.

---

### GAP #4: Remove Prompting Not Implemented

**Location:** `executor.rs:643-644`

```rust
RemoveStep::RmConfig { path, prompt: _ } => {
    // TODO: implement prompting
    format!("rm -f {}", ...)
}
```

**Impact:** Config files are always removed without prompting.

---

## CODE QUALITY ISSUES

### Issue #1: Silent Error Swallowing

Many places use `unwrap_or("")` or `unwrap_or_default()` which silently ignore parsing failures:

- `recipe.rs:311` - Source URL defaults to empty string
- `recipe.rs:333` - Git URL defaults to empty string
- `recipe.rs:375` - Build step args default to empty string
- `recipe.rs:416-441` - Install file paths default to empty strings

**Impact:** Malformed recipes produce empty values instead of errors.

---

### Issue #2: No Recipe Validation

There's no validation that required fields are present:
- A recipe with `(install (to-bin ""))` would try to install an empty path
- A recipe with `(acquire (source ""))` would try to download from empty URL

**Impact:** Runtime errors instead of parse-time validation.

---

### Issue #3: Inconsistent Error Handling

Some parsing functions return `Ok(None)` for missing data, others return `Ok(Some(...))` with empty values:
- `parse_acquire` returns `Ok(None)` if nothing matches
- `parse_build` returns `Ok(None)` if steps is empty
- `parse_install` returns `Ok(None)` if files is empty

But within the parsing, empty strings are silently accepted.

---

## RECOMMENDATIONS

### Priority 1: Fix Critical Dependency Parsing Bug

**Option A:** Change recipe format to match parser (breaking change)
```lisp
; OLD (broken)
(deps (build "meson") (runtime "wayland"))
; NEW
(deps wayland)
(build-deps meson)
```

**Option B:** Update parser to handle nested format (recommended)
```rust
fn parse_deps_action(&mut self, expr: &Expr) -> Result<(), RecipeError> {
    if let Some(tail) = expr.tail() {
        for item in tail {
            match item.head() {
                Some("build") => {
                    self.build_deps = item.tail()
                        .map(|t| t.iter().filter_map(|e| e.as_atom().map(String::from)).collect())
                        .unwrap_or_default();
                }
                Some("runtime") => {
                    self.deps = item.tail()
                        .map(|t| t.iter().filter_map(|e| e.as_atom().map(String::from)).collect())
                        .unwrap_or_default();
                }
                _ => {}
            }
        }
    }
    Ok(())
}
```

### Priority 2: Fix SHA256 and Git Reference Parsing

Parse nested verification and reference blocks:
```rust
fn parse_acquire(expr: &Expr) -> Result<Option<AcquireSpec>, RecipeError> {
    // ... existing code ...
    "source" => {
        let url = item.tail()?.first()?.as_atom()?.to_string();
        let verify = item.tail()
            .and_then(|t| t.get(1))
            .and_then(|v| match v.head()? {
                "sha256" => Some(Verify::Sha256(v.tail()?.first()?.as_atom()?.to_string())),
                _ => None,
            });
        return Ok(Some(AcquireSpec::Source { url, verify }));
    }
}
```

### Priority 3: Add Recipe Validation

Add a `validate()` method to Recipe:
```rust
impl Recipe {
    pub fn validate(&self) -> Result<(), RecipeError> {
        if self.name.is_empty() {
            return Err(RecipeError::MissingName);
        }
        if let Some(AcquireSpec::Source { url, .. }) = &self.acquire {
            if url.is_empty() {
                return Err(RecipeError::InvalidAction("empty source URL".into()));
            }
        }
        // ... more validation
        Ok(())
    }
}
```

### Priority 4: Implement Missing Features

1. `parse_configure` - User creation, directory setup
2. Sandbox execution with landlock/seccomp
3. Update mechanism for packages
4. Lifecycle hooks

---

## AFFECTED RECIPES (ALL 26)

All recipes use the broken nested deps format:
- cmake.recipe, fd.recipe, foot.recipe, grim.recipe, gtk-layer-shell.recipe
- jq.recipe, libinput.recipe, libxkbcommon.recipe, mako.recipe, meson.recipe
- ninja.recipe, pkg-config.recipe, redis.recipe, ripgrep.recipe, seatd.recipe
- slurp.recipe, sway.recipe, swaybg.recipe, swayidle.recipe, swaylock.recipe
- waybar.recipe, wayland.recipe, wayland-protocols.recipe, wl-clipboard.recipe
- wlroots.recipe, wofi.recipe

---

## ADDITIONAL BUGS FOUND (Deep Audit)

### BUG #4: Stack Overflow on Circular Dependencies (SEVERITY: CRITICAL)

**Location:** `levitate.rs:163-226` (`install_with_deps`)

**Problem:** The `installed` HashSet only tracks fully-installed packages. During recursive dependency resolution, packages aren't marked as "in progress", so circular dependencies cause infinite recursion.

**Reproduction:**
```bash
# Create circular deps
echo '(package "a" "1.0" (deps b))' > a.recipe
echo '(package "b" "1.0" (deps a))' > b.recipe
levitate install a  # STACK OVERFLOW
```

**Impact:** Crashes with stack overflow. Could be exploited for DoS.

---

### BUG #5: Empty Package Name/Version Accepted (SEVERITY: MEDIUM)

**Location:** `recipe.rs:177-185`

**Problem:** Empty strings are accepted for name and version:
```rust
let name = list.get(1).and_then(|e| e.as_atom()).ok_or(...)?.to_string();
// "" passes this check
```

**Test:**
```lisp
(package "" "")  ; Accepted with name="" version=""
```

---

### BUG #6: Duplicate Actions Silently Override (SEVERITY: LOW)

**Location:** `recipe.rs:214-292` (`parse_action`)

**Problem:** If a recipe has duplicate actions, the last one wins with no warning:
```lisp
(package "test" "1.0"
  (description "first")
  (description "second"))  ; "second" wins, no warning
```

---

## SECURITY VULNERABILITIES

### SECURITY #1: Path Traversal in Install (SEVERITY: HIGH)

**Location:** `executor.rs:440-455` (`install_file`)

**Problem:** Install source paths are not validated:
```lisp
(install (to-bin "../../../etc/passwd"))
```

The `src` path is joined with `build_dir` without canonicalization, allowing escape.

**Fix:** Canonicalize paths and verify they're within build_dir.

---

### SECURITY #2: No Symlink Protection (SEVERITY: MEDIUM)

**Location:** `executor.rs` (multiple)

**Problem:**
- `build_dir` could be a symlink to sensitive location
- Extracted archives could contain symlinks
- No `O_NOFOLLOW` equivalent protections

---

### SECURITY #3: Potential Shell Injection via PREFIX (SEVERITY: MEDIUM)

**Location:** `executor.rs:806-811` (`expand_vars`)

**Problem:** If `$PREFIX` contains shell metacharacters, they're expanded:
```rust
s.replace("$PREFIX", &self.ctx.prefix.display().to_string())
```

If prefix is `/tmp/$(whoami)`, the shell will execute `whoami`.

**Fix:** Shell-quote the expanded variables or use execvp without shell.

---

## ROBUSTNESS ISSUES

### ROBUSTNESS #1: No Command Timeout

**Location:** `executor.rs:789-803` (`run_cmd`)

**Problem:** Commands can hang forever. No timeout mechanism.

---

### ROBUSTNESS #2: No Output Limits

**Location:** `executor.rs:789-803`

**Problem:** Command output is fully captured. A malicious build could exhaust memory.

---

### ROBUSTNESS #3: Archive Detection Fragile

**Location:** `executor.rs:349-376` (`find_archive`)

**Problem:**
- Returns FIRST matching archive (non-deterministic if multiple)
- No validation that found archive matches expected download
- Race condition between find and extract

---

### ROBUSTNESS #4: Installed DB Race Condition

**Location:** `levitate.rs:104-123`

**Problem:** Multiple levitate processes could corrupt the installed DB:
1. Process A loads DB
2. Process B loads DB
3. Process A saves (with package X)
4. Process B saves (without package X, overwrites)

**Fix:** Use file locking (flock).

---

### ROBUSTNESS #5: Cleanup Keep Pattern Too Broad

**Location:** `executor.rs:706-721` (`cleanup_with_keep`)

**Problem:** Uses prefix matching:
```rust
if !keep.iter().any(|k| name == *k || name.starts_with(k))
```

Keeping "cache" also keeps "cached", "caching", etc.

---

## COMPLETE BUG SUMMARY

| # | Bug | Severity | Location |
|---|-----|----------|----------|
| 1 | Dependency parsing broken | CRITICAL | recipe.rs:245-258 |
| 2 | SHA256 verification not parsed | HIGH | recipe.rs:306-313 |
| 3 | Git references not parsed | MEDIUM | recipe.rs:329-335 |
| 4 | Stack overflow on circular deps | CRITICAL | levitate.rs:163-226 |
| 5 | Empty name/version accepted | MEDIUM | recipe.rs:177-185 |
| 6 | Duplicate actions no warning | LOW | recipe.rs:214-292 |
| S1 | Path traversal in install | HIGH | executor.rs:440-455 |
| S2 | No symlink protection | MEDIUM | executor.rs (multiple) |
| S3 | Shell injection via PREFIX | MEDIUM | executor.rs:806-811 |
| R1 | No command timeout | MEDIUM | executor.rs:789-803 |
| R2 | No output limits | MEDIUM | executor.rs:789-803 |
| R3 | Archive detection fragile | LOW | executor.rs:349-376 |
| R4 | Installed DB race condition | MEDIUM | levitate.rs:104-123 |
| R5 | Cleanup keep too broad | LOW | executor.rs:706-721 |

---

## PRIORITY FIX ORDER

1. **BUG #1**: Dependency parsing (blocks all functionality)
2. **BUG #4**: Circular dependency crash (stability)
3. **SECURITY #1**: Path traversal (security)
4. **BUG #2**: SHA256 parsing (security)
5. **BUG #5**: Empty name/version validation
6. **ROBUSTNESS #4**: File locking for installed DB
7. Remaining issues

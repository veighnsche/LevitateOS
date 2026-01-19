# Documentation Gaps and Open Design Questions

Analysis performed 2026-01-19. This supplements TEAM_043_readme-core-principles.md.

---

## Critical Gaps

### 1. Recipe Binary Not in ISO

**Problem:** The ISO boots to a shell, but `recipe` command doesn't exist.

From `leviso/ARCHITECTURE.md`:
```
## TODO: Recipe Binary Integration

Currently missing from the ISO:
- `/usr/bin/recipe` - The package manager binary
- `/usr/share/recipe/recipes/*.recipe` - Package definitions
```

**Impact:** Breaks the entire `recipe bootstrap /mnt` workflow. Users can't install anything.

**Decision needed:** How should recipe binary be included?
- Pre-built binary in airootfs overlay?
- Cargo build during ISO creation?
- Downloaded at first boot (requires network)?

---

### 2. `recipe bootstrap` Not Implemented

**Documented as:** `recipe bootstrap /mnt` - installs base system like Arch's pacstrap (in leviso/ARCHITECTURE.md)

**Reality:** This command doesn't exist in the codebase.

**What's needed:**
- New subcommand in recipe CLI
- Logic to install base packages to target directory
- Handling of /etc, /var, fstab generation
- Post-install hooks for bootloader

---

### 3. Recipe State Persistence Problem

**Current implementation:** Writes state back to recipe files:
```rhai
// Before install
let installed = false;

// After install (file modified in place)
let installed = true;
let installed_version = "14.1.0";
let installed_at = 1737312000;
let installed_files = ["/usr/bin/rg"];
```

**Problems:**
1. **Git conflicts** - Every install creates a diff
2. **Multi-user race conditions** - Concurrent installs corrupt state
3. **Non-idempotent** - Running twice gives different file contents
4. **Rollback** - No way to restore previous state

**Alternatives to consider:**
- Separate state database (SQLite)
- State directory (`/var/lib/recipe/state/`)
- Keep recipes immutable, state in lockfile

---

### 4. Installer: Docs Viewer vs AI Vision

**What `installer-requirements.md` describes (369 lines):**
- SmolLM3-3B running locally for natural language understanding
- Multi-turn conversation with function calling
- Actions: ListDisks, Partition, Format, Mount, CreateUser, etc.
- ratatui TUI with chat interface

**What's actually implemented (`installer/src/`):**
- TypeScript/Ink docs viewer (NOT Rust)
- Tmux split: bash shell (left) + docs panel (right)
- Navigation: arrow keys, j/k scrolling
- NO LLM, NO function calling, NO disk operations

**The README describes it as:**
```
A tmux-based installer with a split view: native bash shell (left)
for running commands, and a TypeScript TUI docs viewer (right) for guidance.
```

**Decision needed:** These are two completely different visions:
1. **AI-powered conversational installer** (requirements doc) - ambitious, not started
2. **Tmux docs viewer + manual shell** (current code) - simpler, partially working

Which direction? The README accurately describes current code, but `installer-requirements.md` describes something that doesn't exist.

---

### 5. Recipes Submodule is Empty

**Location:** `/home/vince/Projects/LevitateOS/recipes/` (git submodule)

**Contents:** Only a README.md that describes **S-expressions** (wrong format!)

**Actual recipes:** Located in `recipe/examples/` (Rhai scripts, not the submodule)

**Problem:** The `recipes/` submodule appears to be a placeholder that was never populated. The README still describes S-expressions:
```lisp
(package "name" "version"
  (acquire (source "https://..."))
  ...)
```

But actual examples use Rhai:
```rhai
let name = "ripgrep";
let version = "14.1.1";
fn acquire() { download(...); }
fn install() { install_bin("rg"); }
```

**Decision needed:**
1. Delete the `recipes/` submodule and use `recipe/examples/` as the source?
2. Populate the submodule with real `.rhai` recipes?
3. Merge them into a single location?

---

## Documentation vs Implementation Mismatches

### Recipe Format

| Document | Says | Reality |
|----------|------|---------|
| ~~Root README~~ | ~~S-expressions~~ | **FIXED** - Now documents Rhai |
| recipes/README.md (submodule) | S-expressions | **STALE** - Submodule is placeholder |
| recipe/README.md | Rhai scripts | **CORRECT** |
| ~~leviso/ARCHITECTURE.md~~ | ~~S-expressions~~ | **FIXED** - Now documents Rhai |

**Action:** Update recipes/ submodule README or remove submodule

### Installer

| Document | Says | Reality |
|----------|------|---------|
| installer-requirements.md | Full AI system (ratatui, SmolLM3-3B, tool calling) | TypeScript docs viewer only |
| installer/README.md | Tmux + docs viewer | **CORRECT** |
| Root README.md | "AI-powered installer" with SmolLM3-3B | Partially misleading |

**Action:** Add "Status: Future Vision" header to installer-requirements.md

### Recipe Locations

| Location | Contains | Status |
|----------|----------|--------|
| `recipes/` (submodule) | README only, no recipes | **Stale/placeholder** |
| `recipe/examples/` | 17 Rhai example recipes | **Active development** |

**Action:** Consolidate recipe locations, decide canonical source

---

## Missing Documentation

### 1. Recipe Discovery
How does `recipe install foo` find the recipe file?
- Glob pattern? (`recipes/**/*.rhai`)
- Index file?
- Fixed directory?

### 2. Helper Functions
README claims "~35 functions" but doesn't list them all. Actually registered:

**Acquire:** download, copy_files, verify_sha256
**Build:** extract, change_dir, run_cmd/shell
**Install:** install_bin, install_lib, install_man, install_to_dir, rpm_install
**Filesystem:** exists, file_exists, dir_exists, mkdir, rm_files, move_file, symlink, chmod_file
**IO:** read_file, glob_list
**Environment:** get_env, set_env
**Command:** run_output, run_status
**HTTP:** http_get, github_latest_release, github_latest_tag, parse_version
**Process:** exec, exec_output

### 3. Dependency Resolution
- How are cycles detected?
- What happens on version conflicts?
- Can deps specify version constraints?

### 4. Error Recovery
- What if install fails mid-way?
- Are installed files tracked for cleanup?
- Can partial installs be resumed?

### 5. Rocky RPM Selection
- leviso hardcodes RPM list in Rust
- How do users customize base packages?
- Is packages.txt actually read?

---

## Integration Architecture Questions

### Boot Flow

```
[ISO Boot] → [Live Shell] → [???] → [Installed System]
                              ^
                              |
                    How does user run installer?
                    How does recipe get invoked?
```

**Missing pieces:**
1. Auto-start mechanism for installer
2. Recipe binary availability
3. Bootstrap command implementation

### Package Layers

```
Layer 1: Rocky RPMs (extracted by leviso)
         ↓
Layer 2: Recipe packages (sources/binaries)
         ↓
Layer 3: User customization
```

**Questions:**
- Can recipe manage Rocky RPMs, or only its own packages?
- How do layers interact (conflicts, dependencies)?
- Where does system configuration live?

---

## Prioritized Action Items

### P0 - Blocking Everything

1. **Add recipe binary to ISO**
   - Build recipe crate during ISO creation
   - Copy to airootfs/usr/bin/recipe

2. **Implement `recipe bootstrap`**
   - New CLI subcommand
   - Install base packages to target root
   - Generate fstab, configure bootloader

### P1 - Documentation Accuracy

3. **Fix recipes/ submodule**
   - Either populate with real .rhai recipes
   - Or remove submodule and use recipe/examples/ as canonical source

4. **Update installer-requirements.md**
   - Add "Status: Future Vision - Not Yet Implemented" header
   - Current implementation is tmux+docs viewer, not AI

5. **Fix root README.md installer section**
   - Clarify that SmolLM3 is for training data, not runtime
   - Or remove AI claims if direction has changed

### P2 - Design Decisions

6. **Fix recipe state persistence**
   - Choose: database, lockfile, or accept file mutation

7. **Decide installer direction**
   - Full SmolLM3 integration or simpler approach?

8. **Document package layer interaction**
   - How Rocky RPMs + recipe packages coexist

---

## Files Needing Updates

| File | Issue | Action |
|------|-------|--------|
| recipes/README.md | Wrong format (S-expressions) | Update to Rhai or remove submodule |
| installer/installer-requirements.md | Describes unimplemented AI system | Add "Status: Future Vision" header |
| ~~leviso/ARCHITECTURE.md~~ | ~~S-expression refs~~ | **FIXED** |
| recipe/README.md | Complete and accurate | No changes needed |
| ~~README.md~~ | ~~S-expressions, musl~~ | **FIXED** |
| Root README.md | AI installer claims | Clarify SmolLM3 status |

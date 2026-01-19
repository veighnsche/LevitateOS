# TEAM_042: Recipe Codebase Restructure

## Task
Flatten the recipe codebase structure: `engine/` → `core/` + `helpers/`

## Goal
Make it obvious what's helper code (recipe-facing) vs infrastructure (shim).

## Status: COMPLETE ✓
- [x] Create directory structure
- [x] Move core files
- [x] Move helper files
- [x] Create mod.rs files
- [x] Update lib.rs
- [x] Update all import paths
- [x] Delete old directories
- [x] Run tests (all 155+ tests pass)

## Migration Map

| Current | New |
|---------|-----|
| `engine/mod.rs` | `lib.rs` |
| `engine/lifecycle.rs` | `core/lifecycle.rs` |
| `engine/context.rs` | `core/context.rs` |
| `engine/recipe_state.rs` | `core/recipe_state.rs` |
| `engine/deps.rs` | `core/deps.rs` |
| `engine/output.rs` | `core/output.rs` |
| `engine/phases/acquire.rs` | `helpers/acquire.rs` |
| `engine/phases/build.rs` | `helpers/build.rs` |
| `engine/phases/install.rs` | `helpers/install.rs` |
| `engine/util/filesystem.rs` | `helpers/filesystem.rs` |
| `engine/util/io.rs` | `helpers/io.rs` |
| `engine/util/env.rs` | `helpers/env.rs` |
| `engine/util/command.rs` | `helpers/command.rs` |
| `engine/util/http.rs` | `helpers/http.rs` |
| `engine/util/exec.rs` | `helpers/process.rs` |

## Decisions
- Keeping RecipeEngine struct in lib.rs as the main public API
- helpers/mod.rs will have `register_all()` function to wire up all helpers

## Additional Work

### Example Test Recipes (created)
Tests every helper function for integration/e2e testing:
- `examples/test-filesystem.rhai` - mkdir, rm, mv, ln, chmod, exists, file_exists, dir_exists
- `examples/test-io-env.rhai` - read_file, glob_list, env, set_env
- `examples/test-command.rhai` - run, shell, run_output, run_status, exec, exec_output
- `examples/test-http.rhai` - http_get, github_latest_release, github_latest_tag, parse_version
- `examples/test-install.rhai` - install_bin, install_lib, install_man, install_to_dir
- `examples/test-acquire.rhai` - download, copy, extract
- `examples/test-all-helpers.rhai` - comprehensive test of all 32 helpers

### Integration Tests (tests/helpers.rs)
- 12 passing tests that execute example recipes
- 3 ignored tests requiring network access (`cargo test -- --ignored`)
- Tests copy examples to temp directories to avoid state pollution
- Includes inline unit tests for edge cases

### Recipe Validation
Added mandatory validation in `core/lifecycle.rs`:
- **Required variables:** `name`, `version`
- **Required functions:** `acquire()`, `install()`
- Validates types (name/version must be non-empty strings)
- Lists ALL missing items in error message

Example error:
```
Error: Invalid recipe 'bad-recipe' (/path/bad-recipe.rhai):
  - missing required variable: `let name = ...;`
  - missing required variable: `let version = ...;`
  - missing required function: `fn acquire() { ... }`
  - missing required function: `fn install() { ... }`
```

### API Fixes
- `install_to_dir()` now has 2-arg version (no mode) and 3-arg version (with i64 mode)
- Rhai doesn't convert `()` to `Option<u32>`, so needed separate overloads

## Test Count: 262 tests (3 network tests ignored)

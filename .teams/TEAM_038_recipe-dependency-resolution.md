# TEAM_038: Recipe Dependency Resolution

## Status: COMPLETED

## Objective

Implement dependency resolution for Recipe package manager following the research plan:
1. Add `let deps = []` to recipe format ✅
2. Add `recipe install --deps` mode ✅
3. Implement topological sort (pacman-style iterative DFS) ✅
4. Defer version constraints and provides/conflicts (YAGNI) ✅

## Implementation Summary

### New Files
- `src/engine/deps.rs` - Dependency graph and topological sort algorithm

### Modified Files
- `src/engine/mod.rs` - Export deps module
- `src/lib.rs` - Re-export deps module
- `src/bin/recipe.rs` - Add `--deps` flag and `deps` subcommand

### Example Files
- `examples/core-utils.rhai` - Base package (no deps)
- `examples/dev-tools.rhai` - Depends on core-utils
- `examples/myapp.rhai` - Diamond dependency pattern

## Key Design Decisions

### Where does dependency info live?
In the recipe file: `let deps = ["openssl", "zlib"];`

### Who does resolution?
Executor - parse all recipes, build graph, topological sort

### Version constraints?
Start simple: just names. Operators like `>= 1.1` deferred for future.

### Algorithm
Iterative DFS with explicit stack (like pacman/libalpm):
- Uses `NodeState` enum: Unprocessed → Processing → Processed
- Cycle detection: hitting Processing node = cycle
- Post-order traversal: add to result after all children processed

## Usage

```bash
# Show direct dependencies
recipe deps myapp

# Show resolved install order
recipe deps myapp --resolve

# Install with dependencies
recipe install --deps myapp

# Info now shows dependencies
recipe info myapp
```

## Tests

All 221 tests pass (225 total, 4 ignored HTTP tests) including:

### Unit Tests (41 new in deps.rs)
- Graph construction and traversal
- Linear, diamond, and complex dependency patterns
- Cycle detection (self-cycle, two-node, mid-chain)
- Deep chains (5 and 10 levels)
- Wide graphs (5-10 siblings)
- Missing dependencies handling
- Duplicate deps handling
- Filter uninstalled logic
- Package name edge cases

### E2E CLI Tests (12 new)
- `deps` command showing direct dependencies
- `deps --resolve` showing install order
- `install --deps` installing with dependencies
- Diamond pattern resolution
- Skip already installed packages
- Info command showing dependencies

## Future Work (Deferred)

1. Version constraints (`let deps = ["openssl >= 1.1"]`)
2. Provides/virtual packages
3. Conflicts detection
4. Dependency caching for performance

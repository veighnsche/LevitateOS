# TEAM 055: Recipe CLI Refactoring

## Goal
Eliminate code duplication in `recipe/src/bin/recipe.rs` (1405 lines)

## Problems Identified

### 1. Recipe State Access Pattern (15+ occurrences)
```rust
let installed: Option<bool> = recipe_state::get_var(&path, "installed").unwrap_or(None);
let version: Option<String> = recipe_state::get_var(&path, "version").unwrap_or(None);
let installed_version: Option<recipe_state::OptionalString> =
    recipe_state::get_var(&path, "installed_version").unwrap_or(None);
```

Lines: 239-242, 393-395, 453-454, 478-479, 533-536, 576-579, 625-626, 667-669, 944-948, 1022-1028

### 2. Recipe Enumeration Pattern (5 occurrences)
```rust
for entry in std::fs::read_dir(recipes_path)? {
    let entry = entry?;
    let path = entry.path();
    if path.extension().map(|e| e == "rhai").unwrap_or(false) {
        // extract metadata
    }
}
```

Lines: 703-724 (Lock::Update), 896-908 (find_installed_recipes), 938-966 (list_packages), 987-1007 (search_packages)

### 3. Engine Creation (6 occurrences)
```rust
let engine = create_engine(&cli.prefix, cli.build_dir.as_deref(), &recipes_path)?;
```

Lines: 283, 301, 332, 337, 353, 494

## Solution

### Phase 1: Add RecipeMetadata struct
Bundle common recipe state into a single struct with a `load()` method.

### Phase 2: Add enumerate_recipes() iterator
Single function that yields (name, path) pairs for all .rhai files.

### Phase 3: Refactor commands to use new abstractions
Update each command handler to use RecipeMetadata and enumerate_recipes().

## Progress
- [x] Phase 1: RecipeMetadata struct
- [x] Phase 2: enumerate_recipes() iterator
- [x] Phase 3: Refactor commands

## Results

### Changes Made
1. Added `RecipeMetadata` struct (lines 17-32) that bundles all common recipe state queries
2. Added `enumerate_recipes()` iterator (lines 74-80) for iterating over .rhai files
3. Refactored all commands to use these new abstractions:
   - `find_installed_recipes()` - now 8 lines (was 17)
   - `find_upgradable_recipes()` - now 6 lines (was 15)
   - `list_packages()` - now uses RecipeMetadata
   - `search_packages()` - now uses RecipeMetadata
   - `show_info()` - now uses RecipeMetadata
   - Install, Update, Upgrade, Deps, Orphans, Autoremove, Tree, Why, Impact, Lock commands

### Metrics
- **Lines reduced**: 1405 → 1384 (21 lines saved)
- **Duplication eliminated**: 15+ copies of `recipe_state::get_var` pattern → 1 central location
- **All 21 tests pass**

### Key Benefits
- Single source of truth for recipe metadata access
- Easier to add new metadata fields in the future
- More readable command handlers
- Consistent error handling for state access

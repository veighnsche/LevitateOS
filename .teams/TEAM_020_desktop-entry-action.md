# TEAM_020: Add Desktop Entry Action to Recipe System

## Status: Complete

## Goal
Add a new top-level `desktop` action to the recipe system that allows recipes to define desktop entry fields inline, generating a proper `.desktop` file at install time.

## Syntax
```lisp
(package "myapp" "1.0.0"
  (description "My Application")
  (desktop
    (name "My Application")
    (exec "myapp")
    (icon "myapp")
    (categories "Utility;Development")
    (terminal)        ; optional flag - run in terminal
    (comment "Custom comment"))  ; optional, defaults to description
  (install
    (to-bin "myapp")))
```

## Implementation Steps
1. [x] Create team file
2. [x] Add `DesktopSpec` struct in `recipe/src/recipe.rs`
3. [x] Add `pub desktop: Option<DesktopSpec>` to `Recipe` struct
4. [x] Add `"desktop" => ...` case in `parse_action()` method
5. [x] Add `parse_desktop()` method
6. [x] Add `install_desktop()` function in `recipe/src/executor/install.rs`
7. [x] Update executor to call desktop installation after regular install
8. [x] Add unit tests for parsing
9. [x] All tests passing

## Files Modified
- `recipe/src/recipe.rs` - Added DesktopSpec struct, desktop field, parse_desktop() method, tests
- `recipe/src/executor/install.rs` - Added install_desktop() function
- `recipe/src/executor/mod.rs` - Added install_desktop() method call in execute flow
- `recipe/src/lib.rs` - Export DesktopSpec

## Generated Desktop Entry Format
```ini
[Desktop Entry]
Type=Application
Name=My Application
Exec=myapp
Icon=myapp
Comment=My Application description
Categories=Utility;Development;
Terminal=false
```

## Notes
- Desktop files follow freedesktop.org spec
- Install location: `$PREFIX/share/applications/{package-name}.desktop`
- Comment defaults to package description if not specified
- Categories automatically get trailing semicolon if missing
- Both `name` and `exec` are required fields; others are optional

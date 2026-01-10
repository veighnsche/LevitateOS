# Phase 5: Polish & Documentation

## Objective

Clean up, document, and prepare for handoff.

## Tasks

### Step 1: Code Cleanup
- [ ] Remove dead code from old lsh
- [ ] Clean up any temporary workarounds
- [ ] Ensure consistent code style
- [ ] Add TEAM comments for traceability

### Step 2: Documentation
- [ ] Update docs/ARCHITECTURE.md with shell info
- [ ] Create docs/SHELL.md with brush usage guide
- [ ] Document any LevitateOS-specific behavior
- [ ] Update README if needed

### Step 3: Deprecate Old Shell
- [ ] Decide: remove lsh or keep as fallback?
- [ ] If removing: delete crates/userspace/shell/
- [ ] If keeping: rename to lsh-legacy or similar

### Step 4: Update Tests
- [ ] Update golden logs if needed
- [ ] Add shell scripting to CI tests
- [ ] Document test procedures

## Documentation Outline

### docs/SHELL.md
```markdown
# LevitateOS Shell (brush)

## Overview
LevitateOS uses brush, a POSIX/Bash-compatible shell written in Rust.

## Basic Usage
- Commands: type and press Enter
- History: Up/Down arrows
- Tab completion: Tab key
- Exit: `exit` or Ctrl+D

## Scripting
brush is fully Bash-compatible.

### Variables
name="value"
echo $name

### Arrays
arr=(1 2 3)
echo ${arr[@]}

### Conditionals
if [ -f /file ]; then
    echo "exists"
fi

### Loops
for x in 1 2 3; do
    echo $x
done

### Functions
greet() {
    echo "Hello, $1"
}

## References
- brush: https://github.com/reubeno/brush
- Bash Manual: https://www.gnu.org/software/bash/manual/
```

### Step 5: TODO Tracking
- [ ] Add any incomplete work to `TODO.md`
- [ ] Add `TODO(TEAM_XXX)` comments in code for future work
- [ ] Document known limitations

## Handoff Checklist (Rule 10)

Before closing this plan:

- [ ] Project builds cleanly (`cargo xtask build`)
- [ ] All tests pass (`cargo xtask test`)
- [ ] Golden log tests pass
- [ ] Team file updated with final status
- [ ] Remaining TODOs documented
- [ ] docs/ARCHITECTURE.md updated
- [ ] docs/SHELL.md created

## Success Criteria

- [ ] Old shell code removed or archived
- [ ] Documentation complete
- [ ] All tests pass
- [ ] Golden log tests pass
- [ ] Clean handoff to future teams

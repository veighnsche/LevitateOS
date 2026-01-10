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
- [ ] Create docs/SHELL.md with Ion usage guide
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
# LevitateOS Shell (Ion)

## Overview
LevitateOS uses Ion Shell, a modern Rust shell from Redox OS.

## Basic Usage
- Commands: type and press Enter
- History: Up/Down arrows
- Tab completion: Tab key
- Exit: `exit` or Ctrl+D

## Scripting
Ion uses its own syntax (not POSIX/Bash).

### Variables
let name = "value"
echo $name

### Arrays
let arr = [1 2 3]
echo @arr

### Conditionals
if test -f /file
    echo "exists"
end

### Loops
for x in 1 2 3
    echo $x
end

### Functions
fn greet name
    echo "Hello, $name"
end

## References
- Ion Manual: https://doc.redox-os.org/ion-manual/
```

## Success Criteria

- [ ] Old shell code removed or archived
- [ ] Documentation complete
- [ ] All tests pass
- [ ] Clean handoff to future teams

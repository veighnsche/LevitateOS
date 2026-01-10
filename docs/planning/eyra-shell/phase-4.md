# Phase 4: Integration & Testing

## Objective

Integrate brush shell into LevitateOS and verify it works correctly.

## Integration Tasks

### Step 1: Init Integration
- [ ] Update init to spawn brush instead of lsh
- [ ] Verify shell starts on boot
- [ ] Handle shell crash gracefully

### Step 2: Initramfs Integration
- [ ] Add brush binary to initramfs
- [ ] Remove old lsh binary (or keep as fallback)
- [ ] Update build scripts

### Step 3: Test Infrastructure
- [ ] Create bash test scripts for coreutils
- [ ] Create bash test scripts for shell features
- [ ] Integrate with run-test.sh

## Test Cases

### Basic Shell Tests
- [ ] Shell starts and shows prompt
- [ ] Can type and execute commands
- [ ] Backspace/delete work
- [ ] History (up/down arrows) works
- [ ] Tab completion works
- [ ] Ctrl+C interrupts command
- [ ] exit command works

### Scripting Tests (Bash Syntax)
- [ ] Variable assignment: `x="hello"`
- [ ] Variable expansion: `echo $x`
- [ ] If conditionals: `if [ -f /file ]; then echo exists; fi`
- [ ] For loops: `for x in 1 2 3; do echo $x; done`
- [ ] Functions: `greet() { echo hello; }`
- [ ] Script execution: `bash test.sh` or `./test.sh`

### Coreutils Integration Tests
```bash
#!/bin/bash
# test_coreutils.sh
echo "Testing coreutils..."

# Test true/false
if true; then
    echo "true: PASS"
else
    echo "true: FAIL"
fi

if false; then
    echo "false: FAIL"
else
    echo "false: PASS"
fi

# Test pwd
cwd=$(pwd)
echo "pwd: $cwd"

# Test echo
echo "echo: PASS"

# Test cat
echo "test" > /tmp/test.txt
if cat /tmp/test.txt | grep -q test; then
    echo "cat: PASS"
else
    echo "cat: FAIL"
fi

echo "All tests complete"
```

## Success Criteria

- [ ] Shell boots and is usable
- [ ] All basic shell tests pass
- [ ] Bash scripting tests pass
- [ ] Coreutils integration tests pass
- [ ] No regressions in existing functionality
- [ ] Can run existing Linux bash scripts

# Phase 3: Real Coreutils Testing

## Objective
Replace the internal std-only tests with actual coreutils functionality tests.

## Current Test Runner (Inadequate)
```
Test 1: println!... PASS
Test 2: Vec allocation... PASS
Test 3: String operations... PASS
Test 4: Iterator/collect... PASS
Test 5: Box allocation... PASS
Test 6: std::env::args... PASS
```

These only verify the Eyra std library works, NOT the actual utilities.

## Proposed Test Strategy

### Option A: Shell-Based Testing (Recommended)
Modify the shell to run a test script on boot:
```
# /test_coreutils.sh (in initramfs)
echo "Testing coreutils..."

# Test echo
echo "hello" > /tmp/test.txt
cat /tmp/test.txt  # Should output "hello"

# Test filesystem ops
mkdir /tmp/testdir
ls /tmp/testdir     # Should list empty
touch /tmp/testdir/file
ls /tmp/testdir     # Should list "file"
rm /tmp/testdir/file
rmdir /tmp/testdir

# Test other utils
pwd                 # Should output "/"
true && echo "true works"
false || echo "false works"

echo "[COREUTILS_TEST] RESULT: PASSED"
```

### Option B: Native Test Runner (More Complex)
The test runner uses libsyscall directly (not Eyra std) to:
1. Call spawn() syscall for each utility
2. Capture output via pipes
3. Verify expected output

This avoids the circular dependency of testing Eyra with Eyra.

### Decision: Option C (Init-Based Testing)

**Rationale:** The shell doesn't support script execution, so Option A isn't feasible without significant shell changes. Option C is simpler: enhance init to run basic coreutils tests after eyra-test-runner completes. This uses existing libsyscall spawn infrastructure.

**TEAM_378 Update:** Plan revised to match actual shell capabilities.

## Implementation Plan

### Step 1: Enhance eyra-test-runner OR init
Add coreutils verification to existing test flow:
- Spawn `pwd` and verify it exits successfully
- Spawn `true` and verify exit code 0
- Spawn `false` and verify exit code 1
- Spawn `echo test` and verify output

### Step 2: Use libsyscall spawn_args
The kernel's SYS_SPAWN_ARGS syscall works. Use libsyscall::spawn_args to test utilities.

### Step 3: Minimal output verification
Start with exit code verification (simpler than output capture).

## Success Criteria
- [ ] Each coreutil is actually executed during test
- [ ] Output is verified against expected values
- [ ] Filesystem operations are tested (mkdir, rm, etc.)
- [ ] Exit codes are verified (true=0, false=1)

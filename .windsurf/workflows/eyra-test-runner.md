---
description: Run Eyra userspace test runner
---

# Eyra Test Runner Workflow

Run `./run-test.sh` to test all Eyra userspace utilities.

## What it does

1. Builds Eyra utilities (auto-detects source changes)
2. Creates test initramfs with eyra-test-runner
3. Boots LevitateOS in headless QEMU
4. Runs eyra-test-runner which tests std library functionality
5. Reports PASSED/FAILED based on test results

## Commands

```bash
# Run tests (x86_64)
./run-test.sh

# Run tests (aarch64)
./run-test.sh --aarch64
```

## Test Output

```
Test 1: println!... PASS
Test 2: Vec allocation... PASS
Test 3: String operations... PASS
Test 4: Iterator/collect... PASS
Test 5: Box allocation... PASS
Test 6: std::env::args... PASS (argc=0)
Test Summary: 6/6 passed
[TEST_RUNNER] RESULT: PASSED
✅ All OS internal tests passed!
```

## Troubleshooting

- Tests should auto-rebuild when source changes
- If builds seem stale, check that `build_eyra()` in xtask is calling cargo (not skipping)

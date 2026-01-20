# TEAM 057: SSH Command Refactoring

## Goal
Eliminate SSH command building duplication in `installer/xtask/src/vm.rs`

## Problems Identified

### 5 locations building SSH commands with repeated patterns:

1. `wait_for_ssh()` (lines 165-171):
```rust
let mut args = ssh_args();
args.extend([
    "-o".to_string(), "ConnectTimeout=2".to_string(),
    "-o".to_string(), "BatchMode=yes".to_string(),
    "dev@localhost".to_string(),
    "true".to_string(),
]);
```

2. `try_ssh()` (lines 195-201): Same pattern, ConnectTimeout=3

3. `run_ssh()` (lines 212-218): Same pattern, ConnectTimeout=10

4. `ssh()` (line 680-681): Just adds dev@localhost (interactive)

5. `run()` (lines 808-812): Adds -t, dev@localhost, command

### Issues:
- Magic timeout values: 2, 3, 10 with no explanation
- "dev@localhost" repeated 5 times
- BatchMode=yes repeated 3 times
- No clear distinction between probe/check/run timeouts

## Solution

1. Add constants for timeout values with clear names
2. Create `SshCommand` builder to consolidate patterns
3. Simplify each call site

## Progress
- [x] Add timeout constants
- [x] Create SshCommand builder
- [x] Refactor wait_for_ssh()
- [x] Refactor try_ssh()
- [x] Refactor run_ssh()
- [x] Refactor ssh()
- [x] Refactor run()
- [x] Test (27 tests passing)

## Results

### Changes Made
1. Added SSH constants:
   - `SSH_USER` = "dev"
   - `SSH_HOST` = "localhost"
   - `SSH_TIMEOUT_PROBE` = 2 (quick probe during wait_for_ssh loop)
   - `SSH_TIMEOUT_CHECK` = 3 (status check like try_ssh)
   - `SSH_TIMEOUT_RUN` = 10 (command execution)

2. Created `SshCommand` builder (lines 26-106) with fluent API:
   - `.timeout(secs)` - connection timeout
   - `.batch_mode()` - no password prompts, fail on auth issues
   - `.force_pty()` - for interactive commands like tmux
   - `.command(cmd)` - remote command to run
   - `.build()` - returns Command ready to execute
   - `.build_args()` - returns Vec<String> for testing

3. Removed old `ssh_args()` function - replaced by SshCommand builder

4. Refactored all 5 SSH functions to use SshCommand:
   - `wait_for_ssh()` - probe with SSH_TIMEOUT_PROBE
   - `try_ssh()` - check with SSH_TIMEOUT_CHECK
   - `run_ssh()` - execute with SSH_TIMEOUT_RUN
   - `ssh()` - simple interactive (no timeout, no batch mode)
   - `run()` - with force_pty for tmux

### Tests Added
27 unit tests covering:
- SshCommand builder (12 tests)
- SSH constants validation (3 tests)
- Path functions (8 tests)
- Fedora configuration (4 tests)

### Key Benefits
- Magic timeout values (2, 3, 10) now have named constants with documentation
- "dev@localhost" no longer repeated - uses SSH_USER and SSH_HOST constants
- BatchMode=yes centralized in builder
- Clear separation between probe/check/run timeouts
- Easier to add new SSH options (just add builder method)
- Fully testable - build_args() returns Vec<String> for assertions

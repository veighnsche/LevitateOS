# Team 479: Shell and Authentication Workarounds

## Summary

This document describes the workarounds implemented to get a functional shell on LevitateOS serial and graphical consoles. These are temporary development shortcuts that should be properly fixed in production.

## Workarounds Implemented

### 1. PAM Authentication Bypass

**Problem:** PAM autologin fails with "Authentication service cannot retrieve authentication info"

**Error:** `PAM_AUTHINFO_UNAVAIL` returned even with `pam_permit.so` configuration

**Root Cause:** Unknown. Suspected issues:
- NSS module loading in minimal initramfs
- PAM service discovery issues
- util-linux `login` binary behavior with autologin

**Workaround:** Bypass agetty/login/PAM entirely by running shell directly from systemd

**Files Changed:**
- `crates/builder/src/builder/initramfs.rs` - getty@.service and serial-getty@.service

**What We Changed:**
```diff
- ExecStart=-/sbin/agetty --autologin root --keep-baud 115200,38400,9600 %I vt100
+ ExecStart=-/bin/shell-wrapper
```

**Impact:**
- No user authentication (anyone with console access is root)
- No login session management (utmp/wtmp not updated)
- No PAM session setup

**To Fix Properly:**
1. Debug why PAM returns `PAM_AUTHINFO_UNAVAIL` with `pam_permit.so`
2. Ensure NSS modules load correctly (`libnss_files.so.2`)
3. Check if util-linux `login` has special behavior that bypasses our PAM config
4. Consider using a simpler login binary or building login from source

**Reference Implementation:**
- systemd's `vendor/systemd/units/serial-getty@.service.in` does NOT use `--autologin`
- Standard flow: agetty prompts for username → user types username → agetty invokes login → login does PAM auth
- With `--autologin`, agetty passes `-f` to login, which skips `pam_authenticate()` but still calls `pam_acct_mgmt()` and `pam_open_session()`
- Our `pam_acct_mgmt()` might be failing despite pam_permit.so configuration

**Util-linux Login Flow (from `vendor/util-linux/login-utils/login.c`):**
```c
if (!cxt.noauth)      // -f sets noauth=1
    loginpam_auth();  // SKIPPED with -f
loginpam_acct();      // ALWAYS called - pam_acct_mgmt()
loginpam_session();   // ALWAYS called - pam_setcred(), pam_open_session()
```

**Next Investigation Steps:**
1. Check if PAM is finding `/etc/pam.d/login` at all
2. Strace login to see what files it accesses
3. Try without `--autologin` to see if manual login works

---

### 2. Brush Terminal Query Timeout

**Problem:** Brush shell crashes immediately on serial console

**Error:** `ERROR error: input error occurred: The cursor position could not be read within a normal duration`

**Root Cause:** Brush queries terminal cursor position on startup (`[6n` ANSI escape). Serial console via QEMU socket doesn't respond to these queries fast enough (or at all), causing brush to timeout and exit.

**Workaround:** Created `/bin/shell-wrapper` script that:
1. Uses `read` to get user input (no terminal initialization)
2. Passes commands to `brush -c` (non-interactive mode)
3. Loops to provide continuous prompt

**Files Created:**
- `build/initramfs/bin/shell-wrapper`

**Script Contents:**
```bash
#!/bin/sh
cd "$HOME"
while true; do
    printf '%s@%s:%s# ' "$(id -un 2>/dev/null || echo root)" "$(cat /etc/hostname 2>/dev/null || echo levitate)" "$(pwd)"
    read -r cmd || exit 0
    /bin/brush -c "$cmd"
done
```

**Impact:**
- No shell state between commands (variables/aliases lost)
- No job control (can't background processes with &)
- No command history
- No tab completion
- No line editing (arrow keys don't work)

**To Fix Properly:**
1. File bug with brush project about serial console support
2. Add command-line flag to brush for dumb terminal mode
3. Alternatively, include a simpler shell (dash/ash) in initramfs
4. Or patch brush to check if terminal responds before enabling features

---

### 3. Environment Setup via systemd

**Problem:** Shell needs proper PATH and environment

**Workaround:** Set environment variables in systemd service:
```ini
Environment=HOME=/root
Environment=TERM=dumb
Environment=PATH=/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin
Environment=SHELL=/bin/sh
```

**Impact:**
- Environment is hardcoded, not from user's profile
- `/etc/profile` is not sourced

---

## Testing Commands

```bash
# Start VM
cargo run --bin xtask -- vm start

# Send command
cargo run --bin xtask -- vm send "ls /bin"

# View output
cargo run --bin xtask -- vm log

# Stop VM
cargo run --bin xtask -- vm stop
```

## Known Limitations

1. **Serial console only:** Shell-wrapper is optimized for serial console. Graphical console (tty1) uses same workaround but may work better with proper terminal.

2. **No state persistence:** Each command runs in fresh brush instance. To set variables for multiple commands, use: `export FOO=bar; echo $FOO` on same line.

3. **No interactive programs:** Programs that need terminal features (vim, less, top) won't work properly.

## Priority for Fixes

1. **High:** PAM authentication - security concern for production
2. **Medium:** Brush terminal support - usability issue
3. **Low:** Environment sourcing - convenience issue

## Related Files

- `crates/builder/src/builder/initramfs.rs` - Main configuration
- `crates/builder/src/builder/glibc.rs` - Library collection
- `.teams/TEAM_478_authentication-and-testing.md` - Previous team's PAM debugging

## Vendored Reference Implementations

These are source files already in the vendor directory that can help fix issues properly:

### systemd (getty/login services)
- `vendor/systemd/units/serial-getty@.service.in` - Reference serial getty service
- `vendor/systemd/units/getty@.service.in` - Reference virtual console getty service
- `vendor/systemd/units/console-getty.service.in` - Alternative console service
- `vendor/systemd/units/debug-shell.service.in` - Debug shell (useful reference)
- `vendor/systemd/src/getty-generator/getty-generator.c` - How getty services are generated

### util-linux (agetty/login)
- `vendor/util-linux/term-utils/agetty.c` - Terminal getty implementation
- `vendor/util-linux/login-utils/login.c` - Login program (PAM-only version)

### brush (shell)
- `vendor/brush/` - Rust shell implementation
- Check brush issues/PRs for terminal compatibility fixes

## Key Learnings

1. **systemd's reference doesn't use autologin** - For proper production setup, should present login prompt

2. **Even with -f, login still calls PAM** - `pam_acct_mgmt()` and `pam_open_session()` are always called

3. **brush needs terminal query support disabled** - Modern Rust shells assume full terminal capability

4. **PAM module path matters** - Must be in `/lib64/security/` or `/usr/lib64/security/`

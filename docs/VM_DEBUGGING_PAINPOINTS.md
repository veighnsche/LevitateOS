# VM Debugging Pain Points

This document captures pain points encountered while debugging LevitateOS VM issues, to inform future tooling improvements.

## 1. No Interactive Shell

**Problem:** Cannot run commands directly inside the running VM to inspect state.

**Current Workaround:**
- Use `vm send` to send text to serial, but no way to see command output interactively
- Must rely on boot logs and system messages

**Desired Solution:**
- Bidirectional serial communication that allows command execution and output capture
- Example: `vm exec "cat /etc/shadow"` that returns output

**Technical Challenges:**
- Socket-based serial (`-chardev socket`) allows connection but brush shell crashes on cursor position queries
- The shell expects a real terminal with cursor position support

---

## 2. Serial Socket Limitations

**Problem:** Socket-based serial causes terminal-aware programs to crash.

**Symptoms:**
```
error: input error occurred: The cursor position could not be read within a normal duration
```

**Root Cause:**
- brush shell queries cursor position with escape sequences like `[6n`
- Socket doesn't respond to these queries, causing timeout
- Shell crashes, getty restarts in loop

**Potential Solutions:**
1. Use a PTY (pseudo-terminal) instead of socket
2. Configure brush to fall back to "dumb" terminal mode
3. Use a simpler shell (like busybox sh) for debugging
4. Implement a PTY proxy that handles terminal queries

---

## 3. One-Way Debugging

**Problem:** We can only observe output; cannot inspect internal VM state.

**What We Can See:**
- Kernel boot messages
- systemd journal (partial)
- Login prompts and error messages

**What We Cannot See:**
- File contents inside VM
- Process lists
- Memory state
- Library loading errors
- Detailed PAM decisions

**Desired Capabilities:**
- `vm cat /path/to/file` - Read file contents
- `vm ps` - List processes
- `vm ldd /path/to/binary` - Check library dependencies
- `vm strace PID` - Trace system calls

---

## 4. No Direct File Access

**Problem:** Cannot easily inspect or modify files in the running VM.

**Current Approach:**
- Extract CPIO to inspect static contents
- Rebuild entire initramfs for any change
- No way to inspect runtime file state

**Desired Solution:**
- Mount initramfs read-write via 9p virtfs
- Or: Implement file access via QMP/serial
- Or: Include a debug agent in initramfs that responds to commands

---

## 5. PAM Debugging

**Problem:** PAM provides only generic error messages.

**Observed Error:**
```
Authentication service cannot retrieve authentication info
```

**What This Could Mean:**
- Can't read /etc/shadow (permissions, ownership, path)
- Can't find libnss_files.so (wrong path, missing deps)
- Can't parse nsswitch.conf
- SELinux/AppArmor blocking (unlikely in VM)

**Missing Information:**
- Which specific file PAM tried to access
- What path it searched for NSS modules
- What the actual error code was

**Potential Solutions:**
1. Build PAM with debug logging enabled
2. Use `pam_debug.so` or `pam_warn.so` modules
3. Strace the login process
4. Check /var/log/secure (if syslog works)

---

## Recommended Tooling Improvements

### Short-term (Workarounds)

1. **Alternative shell for debugging:**
   - Include busybox or dash as `/bin/debug-sh`
   - These handle dumb terminals better than brush

2. **Boot-time debug script:**
   - Add init script that dumps system state to serial before login
   - Shows: file permissions, library paths, PAM config

3. **PAM verbose mode:**
   ```
   auth required pam_unix.so debug
   ```
   Adds logging (requires syslog or journald)

### Medium-term (New Features)

1. **vm exec command:**
   ```bash
   cargo run -- vm exec "cat /etc/shadow"
   ```
   - Connects to serial, sends command
   - Waits for shell prompt, captures output
   - Requires working shell (may need busybox)

2. **vm inspect command:**
   ```bash
   cargo run -- vm inspect files /etc/
   cargo run -- vm inspect process login
   ```
   - Uses QMP to access VM memory
   - Extracts file listings, process info

3. **Debug initramfs variant:**
   - Includes strace, ldd, busybox
   - Verbose logging enabled
   - `cargo run -- vm start --debug`

### Long-term (Architecture)

1. **9p filesystem passthrough:**
   - Mount host directory in VM
   - Read/write files directly

2. **Debug agent:**
   - Small daemon in initramfs
   - Listens on virtio-serial
   - Responds to inspection commands
   - Returns JSON results

---

## Current Investigation: PAM "cannot retrieve authentication info"

**Verified working:**
- Shadow file owned by root:root (via fakeroot)
- Permissions 0600
- Password hash correct (verified with python crypt)
- PAM modules present: pam_unix.so, pam_permit.so
- NSS config present: /etc/nsswitch.conf
- NSS module present: libnss_files.so.2

**Suspected issues:**
1. glibc may look for libnss in /usr/lib64/ (need symlink?)
2. libnss_files may need additional libraries at runtime
3. NSS initialization may require specific directory structure

**Next steps:**
- Add /usr/lib64 -> ../lib64 symlink
- Or add libnss_files.so.2 to /usr/lib64/ directly
- Test with busybox shell to bypass brush crash issue

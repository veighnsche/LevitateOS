# Phase 2 — Design

## Proposed Solution

Add two new xtask subcommand groups:

```
cargo xtask debug <subcommand>   # VM inspection (memory, registers)
cargo xtask shell <subcommand>   # Shell interaction
```

---

## Command Design

### 1. `cargo xtask debug mem <addr> [--len N] [--arch ARCH]`

**Purpose:** Dump VM memory without stopping execution.

**Implementation:** Use QMP `memsave` command.

```rust
// Pseudocode
fn debug_mem(addr: u64, len: usize) -> Result<()> {
    let mut client = QmpClient::connect("./qmp.sock")?;
    client.execute("memsave", json!({
        "val": addr,
        "size": len,
        "filename": "/tmp/memdump.bin"
    }))?;
    // Read and hexdump the file
}
```

**Output format:**
```
0xffff8000: 48 8b 05 00 00 00 00 48  H......H
0xffff8008: 89 c7 e8 12 34 56 78 90  ....4Vx.
```

---

### 2. `cargo xtask debug regs [--arch ARCH]`

**Purpose:** Dump CPU registers from running VM.

**Implementation:** Use QMP `human-monitor-command` with `info registers`.

```rust
fn debug_regs() -> Result<()> {
    let mut client = QmpClient::connect("./qmp.sock")?;
    let result = client.execute("human-monitor-command", json!({
        "command-line": "info registers"
    }))?;
    println!("{}", result["return"].as_str().unwrap_or(""));
}
```

---

### 3. `cargo xtask shell exec "<command>" [--timeout N] [--arch ARCH]`

**Purpose:** Run a command in the VM shell and return output.

**Implementation:** 
1. Start QEMU headless with serial stdio
2. Wait for shell prompt (`# `)
3. Send command + newline
4. Capture output until next prompt
5. Return output to host

```rust
fn shell_exec(cmd: &str, timeout: u32) -> Result<String> {
    // Start QEMU with stdin/stdout piped
    // Wait for "# " prompt
    // Write cmd + "\n"
    // Read until "# " appears again
    // Return captured output
}
```

---

## Behavioral Decisions

### Q1: Should `debug` commands require a running VM?

**Options:**
- A) Connect to existing QMP socket (requires VM already running)
- B) Start a new VM if not running

**Recommendation:** Option A — require existing VM. Starting a VM is expensive.

### Q2: What if the shell doesn't respond within timeout?

**Options:**
- A) Return error with partial output
- B) Return error with no output
- C) Kill the VM and return error

**Recommendation:** Option A — return partial output for debugging.

### Q3: How to handle multi-line shell output?

**Options:**
- A) Return raw output including escape codes
- B) Strip ANSI codes, return clean text
- C) Return as JSON with structured fields

**Recommendation:** Option B — strip ANSI for clean output.

### Q4: Should `shell exec` start its own VM or attach to running one?

**Options:**
- A) Always start a fresh VM
- B) Connect to running VM via QMP + serial
- C) Provide both modes (`--attach` flag)

**Recommendation:** Option A for simplicity — fresh VM each time.

### Q5: What QMP socket path to use?

**Options:**
- A) Fixed path `./qmp.sock`
- B) Configurable via `--qmp-socket` flag
- C) Auto-detect from running QEMU

**Recommendation:** Option A with Option B as optional override.

---

## API Summary

| Command | Arguments | Output |
|---------|-----------|--------|
| `debug mem` | `<addr>`, `--len`, `--arch` | Hex dump |
| `debug regs` | `--arch` | Register listing |
| `shell exec` | `"<cmd>"`, `--timeout`, `--arch` | Command output |

---

## Open Questions for User

> [!IMPORTANT]
> **Q1:** Should debug commands require a running VM, or auto-start one?

> [!IMPORTANT]  
> **Q2:** For `shell exec`, should it start a fresh VM each time, or attach to an existing one?

> [!NOTE]
> **Q3:** Any other debugging workflows you need besides memory, registers, and shell commands?

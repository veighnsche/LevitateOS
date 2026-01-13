# VM Module

TEAM_326: Created by merging `shell` + `debug` modules for unified VM interaction.

## Commands

| Command | Description |
|---------|-------------|
| `cargo xtask vm start` | Start persistent VM session |
| `cargo xtask vm stop` | Stop VM session |
| `cargo xtask vm send "ls"` | Send keystrokes to running VM |
| `cargo xtask vm exec "ls"` | Execute command in fresh VM (slow) |
| `cargo xtask vm screenshot` | Take screenshot of running VM |
| `cargo xtask vm regs` | Dump CPU registers |
| `cargo xtask vm mem 0x1000` | Dump memory at address |

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Module entry, command definitions |
| `session.rs` | Persistent VM session management |
| `exec.rs` | One-shot command execution |
| `debug.rs` | Register/memory inspection via QMP |

## Session Workflow

```bash
# Start a session
cargo xtask vm start

# Interact with it
cargo xtask vm send "ls"
cargo xtask vm screenshot

# Debug it
cargo xtask vm regs
cargo xtask vm mem 0x40000000

# Stop when done
cargo xtask vm stop
```

## History

- TEAM_323: Created shell exec (ephemeral execution)
- TEAM_324: Created shell session (persistent VM)
- TEAM_325: Created debug tools (regs, mem)
- TEAM_326: Merged into unified vm module

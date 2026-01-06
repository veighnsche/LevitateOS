# EPIC: Interactive Shell & Unix-like Boot Experience

**Phase:** 8b (Continuation of Userspace)  
**Status:** PLANNING  
**Created:** 2026-01-04  
**Prerequisites:** TEAM_073 (Userspace), TEAM_078 (Device MMIO), TEAM_079 (ELF fixes)

---

## Vision

Transform LevitateOS from "Hello from userspace!" into a **real interactive system** where:

1. The GPU terminal shows the **full boot sequence** (like classic Unix)
2. After boot, a **shell prompt** appears
3. User can type commands and see output
4. Basic levbox work (`echo`, `cat`, `ls`)
5. Single-user root access (no login - we ARE root)

**End Goal:** Boot the kernel, see the boot log scroll by on the GPU, then get a `# ` prompt where you can type commands.

---

## Current State (What We Have)

| Component | Status | Notes |
|-----------|--------|-------|
| GPU Terminal | ✅ Works | TEAM_058/059 - renders text, newline fixed |
| Boot Console | ⚠️ Split | `println!` → UART only, GPU shows static banner |
| Userspace Exec | ✅ Works | "Hello from userspace!" runs |
| Syscalls | ✅ Basic | `write(fd, buf, len)`, `exit(code)` |
| ELF Loader | ✅ Works | Loads from initramfs |
| Keyboard Input | ✅ Works | VirtIO keyboard in kernel |

### The Boot Output Problem

**Current behavior:**
- `println!` macro → `console::_print()` → UART only
- GPU terminal shows hardcoded banner: "LevitateOS Terminal v0.1..."
- Boot log (`[BOOT] Stage 1...`, `Heap initialized.`, etc.) NOT on GPU

**Desired behavior:**
- Boot log scrolls on GPU screen (like classic Unix)
- After Stage 3 (GPU init), `println!` should go to BOTH UART and GPU

---

## What We Need (Gap Analysis)

### 1. Dual Console Output (Blocking)
- [ ] Create global terminal reference accessible from `println!`
- [ ] Modify `console::_print()` to write to BOTH UART and GPU terminal (after Stage 3)
- [ ] Remove hardcoded banner, let actual boot log appear on screen

### 2. Shell Binary
- [ ] Create `sh` (or `lsh` - Levitate Shell) userspace binary
- [ ] Read line from stdin
- [ ] Parse command + arguments
- [ ] Execute command (initially: builtins only)
- [ ] Print prompt, repeat

### 3. Syscall Extensions
- [ ] `read(fd, buf, len)` - Read from stdin (keyboard)
- [ ] `getpid()` - Already exists
- [ ] `spawn(path)` or `exec(path)` - Run another program
- [ ] `waitpid(pid)` - Wait for child (if we do spawn)

### 4. Coreutils (Userspace Binaries)
- [ ] `echo` - Print arguments
- [ ] `cat` - Print file contents (needs `open`/`read`/`close`)
- [ ] `ls` - List directory (needs filesystem syscalls)

### 5. Kernel Support
- [ ] Stdin buffer (keyboard → read syscall)
- [ ] Process table (if supporting multiple processes)
- [ ] Basic file descriptor table per process

---

## Milestones

### Milestone 1: "Boot Log on Screen" 🎯
**Deliverable:** Boot output (`[BOOT] Stage 1...`, etc.) scrolls on GPU like classic Unix.

- Wire `println!` to GPU terminal after Stage 3 initialization
- Remove hardcoded banner from `main.rs`
- Boot log appears on screen automatically

### Milestone 2: "Boot to Prompt"
**Deliverable:** After boot log, shell prompt `# ` appears, accepts keyboard input.

- Implement `read()` syscall for stdin
- Create simple shell that prints prompt and echoes input

### Milestone 3: "Echo Chamber"
**Deliverable:** Shell can run `echo hello` and see output.

- Shell parses commands
- Builtin `echo` command
- Or: separate `echo` binary in initramfs

### Milestone 4: "Self-Sufficient Shell"
**Deliverable:** Shell can spawn external programs from initramfs.

- `spawn()` or `exec()` syscall
- Shell executes external binaries
- `exit` builtin to return to... (halt? reboot?)

### Milestone 5: "Real Unix Feel"
**Deliverable:** Multiple levbox, feels like a real system.

- `cat`, `ls` (requires file syscalls)
- `clear` (ANSI escape codes)
- `help` builtin

---

## Architecture Decisions

### Single-User Model
- No login, no users, no permissions beyond kernel/user
- Everything runs as "root" (PID 1 is shell)
- Simplifies everything - add multi-user later

### Shell Strategy
**Option A: Builtin Shell**
- Shell is part of kernel, runs in kernel mode
- Simpler but not "true" userspace

**Option B: Userspace Shell** ✅ (Recommended)
- Shell is an ELF binary in initramfs
- Kernel spawns it as PID 1 after boot
- True Unix model

### Stdin/Stdout Model
- fd 0 = stdin (keyboard buffer)
- fd 1 = stdout (GPU terminal + UART)
- fd 2 = stderr (same as stdout for now)

---

## Non-Goals (Out of Scope)

- Multi-user / login
- Background processes / job control
- Pipes (`|`) and redirection (`>`, `<`)
- Environment variables
- Scripting / conditionals
- Networking commands

---

## Success Criteria

```
[BOOT] Stage 1: Early HAL (SEC)
Heap initialized.
[BOOT] Stage 2: Memory & MMU (PEI)
MMU re-initialized.
...
[BOOT] Stage 5: Userspace
Starting shell...

LevitateOS v0.1
# echo hello world
hello world
# help
Available commands: echo, cat, ls, clear, help, exit
# exit
[KERNEL] Shell exited. Halting.
```

---

## Estimated Effort

| Milestone | Effort | Teams |
|-----------|--------|-------|
| 1. Boot to Prompt | 2-3 sessions | 1-2 teams |
| 2. Echo Chamber | 1 session | 1 team |
| 3. Self-Sufficient | 2-3 sessions | 2 teams |
| 4. Real Unix Feel | 3-4 sessions | 2-3 teams |

**Total:** ~10 team sessions

---

## Next Steps

1. **Immediate:** Fix GPU terminal newline bug (unblocks everything)
2. **Then:** Implement `read()` syscall for keyboard input
3. **Then:** Create minimal shell binary

---

## References

- `@docs/planning/userspace-phase8/` - Original userspace planning
- `@.teams/TEAM_058_gpu_terminal.md` - GPU terminal bugs
- `@.teams/TEAM_073_userspace_execution.md` - Userspace implementation
- `@kernel/src/syscall.rs` - Current syscall handlers

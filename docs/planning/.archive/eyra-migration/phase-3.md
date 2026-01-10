# Phase 3: Migration

**TEAM_362** | Refactor Userspace to Eyra/std  
**Created:** 2026-01-09

---

## 1. Migration Strategy

### 1.1 Order of Migration

| Priority | App | Complexity | Reason |
|----------|-----|------------|--------|
| 1 | cat | Low | Simple PoC |
| 2 | pwd | Low | One syscall |
| 3 | mkdir | Low | One syscall |
| 4 | rmdir | Low | One syscall |
| 5 | rm | Low | Simple file ops |
| 6 | touch | Medium | File creation + timestamps |
| 7 | cp | Medium | File I/O |
| 8 | mv | Medium | Rename + fallback copy |
| 9 | ln | Medium | Link syscalls |
| 10 | ls | High | Directory iteration, formatting |
| 11 | init | Low | Just spawns shell |
| 12 | shell | High | Interactive, command parsing |

### 1.2 Migration Pattern

For each app:
1. Create `apps/<name>/Cargo.toml` with Eyra
2. Create `apps/<name>/src/main.rs` with std code
3. Build with `-Zbuild-std`
4. Test in isolation
5. Test in LevitateOS

---

## 2. App Rewrites

### 2.1 cat (PoC)

**Before (no_std):**
```rust
#![no_std]
#![no_main]
// ~100 lines of manual file handling
```

**After (std):**
```rust
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: cat <file>");
        std::process::exit(1);
    }
    
    for path in &args[1..] {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        print!("{}", contents);
    }
    
    Ok(())
}
```

### 2.2 init

**After (std):**
```rust
use std::process::Command;

fn main() {
    println!("[INIT] PID {} starting...", std::process::id());
    println!("[INIT] Spawning shell...");
    
    // Note: std::process::Command won't work (no fork/exec)
    // Need to use libsyscall::spawn directly
    let shell_pid = unsafe { libsyscall::spawn("shell") };
    
    if shell_pid < 0 {
        eprintln!("[INIT] ERROR: Failed to spawn shell: {}", shell_pid);
    } else {
        println!("[INIT] Shell spawned as PID {}", shell_pid);
    }
    
    loop {
        std::thread::yield_now(); // or custom yield
    }
}
```

### 2.3 shell

**Key Changes:**
- Use `String` instead of `&[u8]`
- Use `std::io::stdin()` for input
- Use `std::io::stdout()` for output
- Keep using `libsyscall::spawn` for process creation

---

## 3. libsyscall Usage

Even with std, some LevitateOS-specific syscalls need libsyscall:

| Syscall | Why Not std |
|---------|-------------|
| `spawn` | LevitateOS custom, no fork+exec |
| `shutdown` | LevitateOS custom |
| `set_foreground` | LevitateOS custom |

### 3.1 libsyscall as Optional Dependency

```toml
[dependencies]
libsyscall = { path = "../libsyscall", optional = true }

[features]
default = ["levitate-syscalls"]
levitate-syscalls = ["libsyscall"]
```

---

## 4. Phase 3 Steps

### Step 1: Migrate Simple Utilities
- cat, pwd, mkdir, rmdir, rm
- ~30 min each

### Step 2: Migrate Medium Utilities
- touch, cp, mv, ln
- ~45 min each

### Step 3: Migrate Complex Utilities
- ls (directory listing, formatting)
- ~1.5 hours

### Step 4: Migrate init
- Simple, mostly spawning shell
- ~30 min

### Step 5: Migrate shell
- Most complex, interactive I/O
- ~2-3 hours

### Step 6: Integration Testing
- Build all apps
- Create initramfs
- Boot and test each command

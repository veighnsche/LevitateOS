# Phase 2 — Structural Extraction

**Parent:** Userspace Architecture Refactor  
**File:** `docs/planning/userspace-refactor/phase-2.md`  
**Depends On:** Phase 1 complete

---

## Critical Technical Considerations

> [!CAUTION]
> **Three Landmines Identified During Review**
> 
> 1. **Panic Handler Trap** — `#[panic_handler]` must be in binaries, not library
> 2. **Linker Script Scope** — Use `build.rs` per-crate, not global `.cargo/config.toml`
> 3. **Exec Syscall Missing** — No `sys_exec` exists; init cannot exec into shell yet

---

## 1. Target Design

### 1.1 New Directory Structure
```
userspace/
├── Cargo.toml              # [workspace] members = ["libsyscall", "init", "shell"]
├── .cargo/
│   └── config.toml         # target = aarch64-unknown-none (NO linker args here)
├── libsyscall/
│   ├── Cargo.toml          # [lib] crate, no_std
│   └── src/lib.rs          # Syscalls + common_panic_handler() (NOT #[panic_handler])
├── init/
│   ├── Cargo.toml          # [[bin]], depends on libsyscall
│   ├── build.rs            # Linker script: -Tinit/link.ld
│   ├── link.ld
│   └── src/main.rs         # PID 1: banner, exec shell
└── shell/
    ├── Cargo.toml          # [[bin]], depends on libsyscall
    ├── build.rs            # Linker script: -Tshell/link.ld
    ├── link.ld
    └── src/main.rs         # Shell logic + #[panic_handler]
```

### 1.2 Dependency Graph (Panic Handler Pattern)

```
    init (Binary)                 shell (Binary)
         │                              │
    #[panic_handler]              #[panic_handler]
         │                              │
         └──────────┬───────────────────┘
                    ▼
           common_panic_handler()
                    │
                    ▼
              libsyscall (Library)
                    │
                    ▼
              sys_write, sys_exit
```

### 1.3 Responsibility Breakdown

| Crate | Responsibility |
|-------|----------------|
| `libsyscall` | Syscall wrappers, `print!`/`println!` macros, `common_panic_handler()` |
| `init` | PID 1, boot banner, `#[panic_handler]`, exec shell (Phase 8c: spawn) |
| `shell` | Interactive shell, builtins, `#[panic_handler]` |

---

## 2. Extraction Strategy

### 2.1 What Gets Extracted

**To `libsyscall/src/lib.rs`:**
- Syscall constants: `SYS_READ`, `SYS_WRITE`, `SYS_EXIT`, `SYS_GETPID`, `SYS_SBRK`
- Syscall functions: `read()`, `write()`, `exit()`
- `print!` and `println!` macros
- `pub fn common_panic_handler(info: &PanicInfo) -> !` (shared logic)

**Stays in `shell/src/main.rs`:**
- `#[panic_handler]` attribute (calls `common_panic_handler`)
- Shell-specific: `read_line()`, `trim()`, `execute()`, `_start()`

**New `init/src/main.rs`:**
- `#[panic_handler]` attribute (calls `common_panic_handler`)
- Print boot banner
- **For now:** Execute shell logic inline (no exec syscall exists)
- **Phase 8c:** Call `sys_exec("shell")` when implemented

---

## 3. Phase 2 Steps

### Step 1: Create Userspace Workspace

**File:** `userspace/Cargo.toml`
```toml
[workspace]
members = ["libsyscall", "init", "shell"]
resolver = "2"

[profile.release]
panic = "abort"
opt-level = "z"
lto = false
```

**File:** `userspace/.cargo/config.toml`
```toml
[build]
target = "aarch64-unknown-none"

# NO linker args here — handled by per-crate build.rs
```

### Step 2: Create libsyscall Crate

**File:** `userspace/libsyscall/Cargo.toml`
```toml
[package]
name = "libsyscall"
version = "0.1.0"
edition = "2021"

[lib]
```

**File:** `userspace/libsyscall/src/lib.rs`
```rust
#![no_std]

pub mod syscall;  // SYS_READ, SYS_WRITE, etc.

pub use syscall::*;

use core::panic::PanicInfo;

/// Shared panic handler logic — call from #[panic_handler] in binaries
pub fn common_panic_handler(_info: &PanicInfo) -> ! {
    // Use sys_write directly to avoid potential recursion
    let msg = b"PANIC!\n";
    syscall::write(2, msg);
    syscall::exit(1);
}

#[macro_export]
macro_rules! print { ... }

#[macro_export]
macro_rules! println { ... }
```

### Step 3: Create init Crate

**File:** `userspace/init/build.rs`
```rust
fn main() {
    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rustc-link-arg=-Tlink.ld");
}
```

**File:** `userspace/init/src/main.rs`
```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libsyscall::{println, common_panic_handler};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("[INIT] LevitateOS init v0.1");
    println!("[INIT] Starting shell...");
    
    // TODO(TEAM_118): Call sys_exec("shell") when implemented (Phase 8c)
    // For now, init IS the shell (embedded shell logic)
    shell_main();
}
```

### Step 4: Create shell with Per-Crate build.rs

**File:** `userspace/shell/build.rs`
```rust
fn main() {
    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rustc-link-arg=-Tlink.ld");
}
```

**File:** `userspace/shell/src/main.rs`
```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libsyscall::{println, common_panic_handler};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

// ... shell logic (read_line, execute, etc.)
```

---

## 4. Interim Strategy (No Exec Syscall)

Since `sys_exec` does not exist yet:

**Option A: Init embeds shell logic**
- `init/src/main.rs` imports shell functions and runs them
- No separate `shell` binary needed initially
- Simpler, works now

**Option B: Kernel still runs "shell" directly**
- Keep kernel running `run_from_initramfs("shell")`
- `init` binary exists but is not used yet
- Defer full init→shell flow to Phase 8c

**Recommended:** Option B — keeps refactor focused on code organization

---

## 5. Exit Criteria

- [ ] `userspace/Cargo.toml` workspace exists
- [ ] `libsyscall/` compiles independently (no `#[panic_handler]`)
- [ ] `shell/` compiles with `build.rs` linker config and its own `#[panic_handler]`
- [ ] Shell uses `libsyscall` instead of inline syscall module
- [ ] `cargo xtask test all` passes
- [ ] Kernel still runs shell directly (init deferred to Phase 8c)

//! TEAM_169: Environment and argument access for LevitateOS userspace.
//!
//! Provides access to command-line arguments and environment variables.
//!
//! Per Phase 2 Q5 decision: Stack-based argument passing (Linux ABI compatible).
//!
//! ## Stack Layout at _start
//! ```text
//! SP -> argc
//!       argv[0]
//!       argv[1]
//!       ...
//!       argv[argc-1]
//!       NULL
//!       envp[0]
//!       envp[1]
//!       ...
//!       NULL
//! ```

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ffi::{c_char, CStr};

/// TEAM_213: Cached arguments (parsed once at startup).
static ARGS: spin::Once<Vec<String>> = spin::Once::new();

/// TEAM_213: Cached environment variables (parsed once at startup).
static ENV_VARS: spin::Once<Vec<String>> = spin::Once::new();

/// TEAM_217: Cached auxiliary vector (parsed once at startup).
static AUXV: spin::Once<Vec<(usize, usize)>> = spin::Once::new();

/// TEAM_169: Initialize arguments from the stack pointer.
///
/// This should be called once at the very start of _start,
/// before the stack pointer is modified.
///
/// # Safety
/// * Must be called exactly once at program start
/// * `sp` must point to the argc value on the stack
/// * Stack layout must match the Linux ABI
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe fn init_args(sp: *const usize) {
    if ARGS.is_completed() {
        return; // Already initialized
    }

    let mut args = Vec::new();
    let mut envs = Vec::new();
    let mut auxv = Vec::new();

    // Read argc
    let argc = unsafe { *sp };

    // TEAM_185: Sanity check argc to prevent reading invalid memory
    const MAX_ARGC: usize = 4096;
    let argc = argc.min(MAX_ARGC);

    // Read argv pointers (starts at sp + 1)
    let argv_base = unsafe { sp.add(1) };
    for i in 0..argc {
        let arg_ptr = unsafe { *argv_base.add(i) } as *const u8;
        if !arg_ptr.is_null() {
            if let Ok(cstr) = unsafe { CStr::from_ptr(arg_ptr as *const c_char) }.to_str() {
                args.push(String::from(cstr));
            }
        }
    }

    // Read envp pointers (starts after argv NULL terminator)
    let envp_base = unsafe { argv_base.add(argc + 1) };
    let mut env_idx = 0;
    // TEAM_185: Limit envp iteration to prevent infinite loop if NULL is missing
    const MAX_ENVP: usize = 4096;
    while env_idx < MAX_ENVP {
        let env_ptr = unsafe { *envp_base.add(env_idx) } as *const u8;
        if env_ptr.is_null() {
            break;
        }
        if let Ok(cstr) = unsafe { CStr::from_ptr(env_ptr as *const c_char) }.to_str() {
            envs.push(String::from(cstr));
        }
        env_idx += 1;
    }

    // TEAM_217: Read auxiliary vector (starts after envp NULL terminator)
    let auxv_base = unsafe { envp_base.add(env_idx + 1) };
    let mut aux_idx = 0;
    const MAX_AUXV: usize = 64;
    while aux_idx < MAX_AUXV {
        let type_ptr = unsafe { auxv_base.add(aux_idx * 2) };
        let val_ptr = unsafe { auxv_base.add(aux_idx * 2 + 1) };
        let a_type = unsafe { *type_ptr };
        let a_val = unsafe { *val_ptr };
        
        if a_type == 0 { // AT_NULL
            break;
        }
        auxv.push((a_type, a_val));
        aux_idx += 1;
    }

    ARGS.call_once(|| args);
    ENV_VARS.call_once(|| envs);
    AUXV.call_once(|| auxv);
}

/// TEAM_217: Get an auxiliary vector entry by type.
pub fn get_auxv(a_type: usize) -> Option<usize> {
    AUXV.get().and_then(|v| {
        for (t, val) in v {
            if *t == a_type {
                return Some(*val);
            }
        }
        None
    })
}

/// TEAM_169: Get command-line arguments.
///
/// Returns an iterator over the program's arguments.
///
/// # Example
/// ```rust
/// for arg in ulib::env::args() {
///     println!("Arg: {}", arg);
/// }
/// ```
pub fn args() -> Args {
    Args {
        inner: ARGS.get().map(|v| v.iter()),
    }
}

/// TEAM_169: Get the number of command-line arguments.
pub fn args_len() -> usize {
    ARGS.get().map(|v| v.len()).unwrap_or(0)
}

/// TEAM_169: Get a specific argument by index.
pub fn arg(index: usize) -> Option<&'static str> {
    ARGS.get().and_then(|v| v.get(index)).map(|s| s.as_str())
}

/// TEAM_169: Iterator over command-line arguments.
pub struct Args {
    inner: Option<core::slice::Iter<'static, String>>,
}

impl Iterator for Args {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next().cloned()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            Some(iter) => iter.size_hint(),
            None => (0, Some(0)),
        }
    }
}

impl ExactSizeIterator for Args {}

/// TEAM_169: Get environment variables.
///
/// Returns an iterator over environment variable strings (in "KEY=VALUE" format).
pub fn vars() -> Vars {
    Vars {
        inner: ENV_VARS.get().map(|v| v.iter()),
    }
}

/// TEAM_169: Get the number of environment variables.
pub fn vars_len() -> usize {
    ENV_VARS.get().map(|v| v.len()).unwrap_or(0)
}

/// TEAM_169: Get an environment variable by name.
pub fn var(name: &str) -> Option<&'static str> {
    let prefix_len = name.len();
    ENV_VARS.get().and_then(|vars| {
        for var in vars {
            if var.starts_with(name) && var.as_bytes().get(prefix_len) == Some(&b'=') {
                return Some(&var[prefix_len + 1..]);
            }
        }
        None
    })
}

/// TEAM_169: Iterator over environment variables.
pub struct Vars {
    inner: Option<core::slice::Iter<'static, String>>,
}

impl Iterator for Vars {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        // TEAM_185: Use loop instead of recursion to avoid stack overflow
        // on many consecutive malformed env vars
        loop {
            let var_str = self.inner.as_mut()?.next()?;
            // Split on first '='
            if let Some(eq_pos) = var_str.find('=') {
                let key = var_str[..eq_pos].to_string();
                let value = var_str[eq_pos + 1..].to_string();
                return Some((key, value));
            }
            // Malformed env var (no '='), continue to next
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // TEAM_185: Add size_hint - upper bound is inner length,
        // lower bound is 0 since we skip malformed entries
        match &self.inner {
            Some(iter) => (0, Some(iter.len())),
            None => (0, Some(0)),
        }
    }
}

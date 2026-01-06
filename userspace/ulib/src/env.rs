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

/// TEAM_169: Cached arguments (parsed once at startup).
static mut ARGS: Option<Vec<String>> = None;

/// TEAM_169: Cached environment variables (parsed once at startup).
static mut ENV_VARS: Option<Vec<String>> = None;

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
    if unsafe { ARGS.is_some() } {
        return; // Already initialized
    }

    let mut args = Vec::new();
    let mut envs = Vec::new();

    // Read argc
    let argc = unsafe { *sp };

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
    loop {
        let env_ptr = unsafe { *envp_base.add(env_idx) } as *const u8;
        if env_ptr.is_null() {
            break;
        }
        if let Ok(cstr) = unsafe { CStr::from_ptr(env_ptr as *const c_char) }.to_str() {
            envs.push(String::from(cstr));
        }
        env_idx += 1;
    }

    unsafe {
        ARGS = Some(args);
        ENV_VARS = Some(envs);
    }
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
        inner: unsafe { ARGS.as_ref().map(|v| v.iter()) },
        index: 0,
    }
}

/// TEAM_169: Get the number of command-line arguments.
pub fn args_len() -> usize {
    unsafe { ARGS.as_ref().map(|v| v.len()).unwrap_or(0) }
}

/// TEAM_169: Get a specific argument by index.
pub fn arg(index: usize) -> Option<&'static str> {
    unsafe { ARGS.as_ref().and_then(|v| v.get(index)).map(|s| s.as_str()) }
}

/// TEAM_169: Iterator over command-line arguments.
pub struct Args {
    inner: Option<core::slice::Iter<'static, String>>,
    index: usize,
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
        inner: unsafe { ENV_VARS.as_ref().map(|v| v.iter()) },
    }
}

/// TEAM_169: Get the number of environment variables.
pub fn vars_len() -> usize {
    unsafe { ENV_VARS.as_ref().map(|v| v.len()).unwrap_or(0) }
}

/// TEAM_169: Get an environment variable by name.
pub fn var(name: &str) -> Option<&'static str> {
    let prefix_len = name.len();
    unsafe {
        ENV_VARS.as_ref().and_then(|vars| {
            for var in vars {
                if var.starts_with(name) && var.as_bytes().get(prefix_len) == Some(&b'=') {
                    return Some(&var[prefix_len + 1..]);
                }
            }
            None
        })
    }
}

/// TEAM_169: Iterator over environment variables.
pub struct Vars {
    inner: Option<core::slice::Iter<'static, String>>,
}

impl Iterator for Vars {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let var_str = self.inner.as_mut()?.next()?;
        // Split on first '='
        if let Some(eq_pos) = var_str.find('=') {
            let key = var_str[..eq_pos].to_string();
            let value = var_str[eq_pos + 1..].to_string();
            Some((key, value))
        } else {
            // Malformed env var, skip
            self.next()
        }
    }
}

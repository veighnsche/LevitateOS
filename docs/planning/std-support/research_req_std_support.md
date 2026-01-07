# Research Request: Rust `std` Compatibility & POSIX Requirements

## Objective
Identify the **exact, exhaustive list** of system calls, data structures, and ABI conventions required to port the Rust standard library (`std`) to a new operating system ("LevitateOS"). The goal is to enable "Platform Support" (compiling `std` apps like `uutils/levbox` directly).

## Scope
1.  **Rust `std::sys` Interface**:
    - What exact functions must be implemented in the OS-specific backend of `std`? (e.g., `fs`, `io`, `net`, `thread`, `time`, `process`).
    - What are the required `extern "C"` symbols?

2.  **Minimal POSIX Baseline**:
    - What is the absolute minimum set of POSIX syscalls required to run `sh`, `ls`, and `cat`?
    - **Data Structures**: Exact byte layout for `stat`, `dirent`, `timespec`, `sockaddr`.
    - **Constants**: Values for `O_RDONLY`, `O_CREAT`, errno codes (`EPERM`, `ENOENT`), signal numbers.

3.  **Reference Implementations**:
    - **Redox OS**: How does `redox_syscall` map to `std`?
    - **Linux**: What is the AArch64 ABI for these specific calls?
    - **Relibc**: How does a Rust-written libc implement these interfaces on top of syscalls?

## Specific Questions to Answer
- [ ] **Memory**: Does `std` require `mmap` or is `sbrk` sufficient for the default global allocator?
- [ ] **Threading**: exact interface for `thread::spawn` (e.g., `clone` vs `pthread_create` shim).
- [ ] **TLS**: How is Thread Local Storage initialized on AArch64 for `std`?
- [ ] **IO**: What are the specific `ioctl` requests needed for a defined TTY (for interactive apps)?
- [ ] **Filesystem**: What is the expected behavior of `open` with `O_CLOEXEC`?
- [ ] **Startup**: What does `crt0` (startup code) need to pass to `main` (env vars, aux vectors)?

## Output
A specification document (`docs/specs/userspace-abi.md`) containing:
- Complete list of required syscalls + ID + Arguments.
- Struct definitions in Rust (repr(C)).
- Constants table.
- Initialization sequence for the runtime.

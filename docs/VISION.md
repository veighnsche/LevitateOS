# LevitateOS Vision

## 🎯 Mission Statement
To build a **modern, secure, and performant** operating system written in Rust that provides **seamless binary compatibility with the Linux ABI**.

LevitateOS aims to prove that a clean-slate kernel, built with modern language guarantees (Rust), can support the vast existing ecosystem of Linux applications without sacrificing safety or architectural integrity.

## 🏛️ Core Principles

1. **Safety by Default**: Leverage Rust's ownership and type system to enforce memory safety and eliminate entire classes of bugs (e.g., Use-After-Free, Data Races) at compile time.
2. **Linux ABI Compatibility**: Prioritize compatibility with the Linux system call interface. This allows running unmodified Linux binaries (starting with static Rust applications like `uutils`) and enables the use of the standard Rust `std` library.
3. **Modern Pure-Rust Userspace**: Utilize the [Eyra](https://github.com/sunfishcode/eyra) ecosystem (via `rustix` and `linux-raw-sys`) to provide a Linux-compatible runtime that is entirely C-free.
4. **Modular "Worse is Better" Architecture**: Prioritize simple, verifiable implementations over "perfect" but complex ones. Follow the rule of simplicity (Rule 20).
5. **Silence is Golden**: The kernel should be silent in success and loud in failure (Rule 4).
6. **Modern Hardware First**: Targets modern architectures (AArch64, x86_64) and hardware (Pixel 6, Intel NUC) with a focus on energy efficiency and scalability.

## 🚀 Long-Term Goal
The ultimate milestone for LevitateOS is to **port and run the complete Rust Standard Library (`std`)**, enabling developers to build and run complex, production-grade Rust applications natively on LevitateOS as if they were running on Linux.

## 🛠️ Strategy
- **Phase 1-14 (Foundation)**: Establish the HAL, MMU, Multitasking, and VFS.
- **Phase 15-17 (Compatibility)**: Implement the Linux syscall layer and TTY subsystem.
- **Phase 18-20 (Security & Multi-user)**: Add identity, authentication, and kernel hardening.
- **Terminal Support (Phase 16)**
   - Implement `termios` and `ioctl(TCGETS/TCSETS)` for TTY control.
- **Toolchain & Libc (The Eyra Path)**
   - Leverage [Eyra](https://github.com/sunfishcode/eyra) as the primary userspace runtime to achieve a pure Rust, C-library-free environment.
   - Port Rust `std` to `levitateos` target by utilizing Eyra's Linux ABI compatibility layers.

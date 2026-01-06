# repro_crash

Diagnostic tool for LevitateOS kernel and userspace verification.

## Overview

`repro_crash` is a minimal userspace application designed to intentionally trigger specific failure conditions. It is used during development to verify that the kernel correctly handles exceptions, crashes, and invalid system calls without compromising system stability.

## Test Cases

- **Illegal Instruction**: Executes an invalid opcode to verify SIGILL/Exception handling.
- **Data Abort**: Accesses unmapped or restricted memory to verify page fault handling.
- **Stack Overflow**: Uses recursion to exceed the stack limit.
- **Invalid Syscall**: Calls a non-existent syscall number to verify kernel sanitization.

## Usage

Typically launched from the shell to verify a fix or a new security feature:

```bash
/bin/repro illegal_instr
```

## Implementation

The binary is carefully linked with a dedicated `link.ld` and `build.rs` to ensure it has a predictable memory layout for crash analysis.

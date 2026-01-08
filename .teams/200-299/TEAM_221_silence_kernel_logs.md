# TEAM_221 Silence Kernel Logs

## Goal
Remove verbose logging from the kernel to adhere to the "Silence is Golden" rule.

## Context
The user observed excessive logging when running `cat`, including:
- `[SYSCALL] spawn_args`
- `[SPAWN]`
- `[MMU]`
- `[ELF]`
- `[TASK]`

These logs clutter the output and violate the design principle that "Silence implies success".

## Plan
1.  Search for the log strings in the kernel codebase.
2.  Replace `println!` or unconditional logs with `log::debug!` or `log::trace!`.
3.  Verify that the default log level is appropriate (likely `warn` or `info` for crucial boot messages, but definitely not these debug messages).
4.  Ensure `cat` (and other userspace apps) output remains clean.

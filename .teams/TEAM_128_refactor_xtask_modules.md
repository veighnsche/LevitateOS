# TEAM_128: Refactor xtask Modules

## Goals
- Split `xtask/src/main.rs` into logical modules:
  - `build.rs`: Build logic
  - `run.rs`: QEMU/Run logic
  - `image.rs`: Disk image logic
  - `clean.rs`: Clean logic
- Implement VM management style CLI.

## Log
- **[current]** Starting refactor.

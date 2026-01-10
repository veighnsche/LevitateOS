# TEAM_376: Refactor Eyra Coreutils Structure

## Problem Statement

The Eyra coreutils workspace has severe structural issues:

### Issue 1: Duplicate Target Folders (5.1GB wasted)
Each utility has its own `target/` folder from when they were standalone projects:
- `cat/target/` - 802MB
- `cp/target/` - 329MB
- `echo/target/` - 346MB
- ... (15 utilities × ~350MB each = **5.1GB total**)

These are stale - the workspace now uses `eyra/target/` at the root.

### Issue 2: Duplicate Cargo.lock and .cargo files
Each utility has its own:
- `Cargo.lock` - Should use workspace lock
- `.cargo/` folder - Should use workspace config

### Issue 3: Test Runner Only Tests std Internals
Current eyra-test-runner tests:
- println! works
- Vec allocation
- String operations
- Iterator/collect
- Box allocation
- std::env::args

But it does NOT test actual coreutils functionality like:
- `cat /file` actually reads files
- `echo hello` outputs text
- `ls /` lists directories
- `mkdir /test` creates directories

## Refactor Goals

1. **Clean folder structure** - Remove stale target/, Cargo.lock, .cargo from each utility
2. **Shared workspace config** - Single .cargo/config.toml at workspace root
3. **Real coreutils tests** - Test actual utility behavior via syscalls

## Planning Location
`docs/planning/eyra-coreutils-refactor/`

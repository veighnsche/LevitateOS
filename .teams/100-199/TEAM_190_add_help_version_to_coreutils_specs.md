# TEAM_190: Add --help and --version to Coreutils Specs

**Date**: 2026-01-06
**Status**: ✅ Complete

## Task

Add accurate `--help` and `--version` option documentation to all levbox specs based on GNU levbox.

## Research Method

Ran actual GNU levbox commands to get accurate output:
- `/usr/bin/cat --help`, `/usr/bin/cat --version`
- `/usr/bin/cp --help`, `/usr/bin/ls --help`, etc.

## Files Updated

All 10 levbox spec files in `docs/specs/levbox/`:

| File | Options Added |
|------|---------------|
| `cat.md` | `--help`, `--version` |
| `cp.md` | `-v`, `--verbose`, `--help`, `--version` |
| `ln.md` | `--symbolic`, `--force`, `--no-dereference`, `--verbose`, `--help`, `--version` |
| `ls.md` | `--all`, `--almost-all`, `--human-readable`, `--classify`, `--recursive`, `--help`, `--version` |
| `mkdir.md` | `--parents`, `--mode`, `--verbose`, `--help`, `--version` |
| `mv.md` | `--force`, `--interactive`, `--verbose`, `--help`, `--version` |
| `pwd.md` | `--logical`, `--physical`, `--help`, `--version` |
| `rm.md` | `--force`, `-I`, `--recursive`, `--dir`, `--verbose`, `--help`, `--version` |
| `rmdir.md` | `--parents`, `--verbose`, `--ignore-fail-on-non-empty`, `--help`, `--version` |
| `touch.md` | `--no-create`, `--reference`, `--date`, `--help`, `--version` |

## Changes Per File

Each spec now includes:

1. **Options table** - Updated with long option forms (e.g., `-f`, `--force`)
2. **Help and Version Output section** - Accurate example output matching GNU levbox format:
   ```
   ### `utility --help`
   
   ```
   Usage: utility [OPTION]... [ARG]...
   Description...
   
     -o, --option    description
         --help      display this help and exit
         --version   output version information and exit
   ```
   
   ### `utility --version`
   
   ```
   utility (LevitateOS levbox) 0.1.0
   ```
   ```

## Handoff

- [x] All 10 spec files updated
- [x] Options based on actual GNU levbox 9.7 output
- [x] Consistent format across all specs
- [x] Team file created

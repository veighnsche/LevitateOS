# Team 222: Debug Shell Input Regression

## Goal
Fix the bug where `.` and `-` characters cannot be typed in the shell.
Add a regression test to prevent recurrence.

## Context
- User noticed this after I silenced kernel logs.
- `cat test.txt` fails because `.` cannot be typed.
- `cat --help` fails because `-` cannot be typed.

## Investigation Notes
- Need to check `levbox/src/bin/sh.rs` for input filtering.
- Need to check `kernel` keyboard handling if `sh` relies on it.

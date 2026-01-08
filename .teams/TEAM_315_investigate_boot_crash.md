# TEAM_315: Investigate Boot Crash After GDT/IDT/ELF Changes

## Bug Report

**Symptom:** Kernel crashes immediately on boot after changes to:
1. ELF parsing (replaced hand-rolled with goblin crate)
2. GDT/IDT (attempted to replace with x86_64 crate, then reverted)

**Output:** `SMGPCLEHXMRjka` - appears to be garbage/random characters

**Expected:** Normal boot sequence with diagnostic characters 'a' through 'h'

## Investigation Status

- [ ] Reproduce the bug
- [ ] Form hypotheses
- [ ] Test hypotheses
- [ ] Identify root cause
- [ ] Fix

## Hypotheses

TBD after reproduction

## Findings

TBD

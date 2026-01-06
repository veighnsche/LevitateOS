# TEAM_212: Investigate Data Abort on `cat --help`

## Bug Report

**Symptom:** Running `cat --help` in lsh causes a Data Abort exception.

**Error Details:**
```
*** USER EXCEPTION ***
Exception Class: 0x24
ESR: 0x000000009200004f
ELR (fault address): 0x00000000000104e8
Type: Data Abort
Terminating user process.
```

**Environment:** LevitateOS Shell (lsh) v0.1

**Reproduction:**
1. Boot LevitateOS
2. Enter shell
3. Run `cat --help`
4. Observe Data Abort

## Analysis

### Exception Decoding
- **Exception Class 0x24** = Data Abort from lower EL (userspace)
- **ESR 0x9200004f** = Syndrome register value
  - ISS bits indicate: synchronous external abort, level 3 translation fault
- **ELR 0x104e8** = Instruction that caused the fault (in userspace at low address)

## Hypotheses

(To be filled during investigation)

## Root Cause

(To be determined)

## Resolution

(To be determined)

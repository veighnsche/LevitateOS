# KNOWLEDGE: Stop. Read. Then Act.

**Date:** 2026-01-21
**Severity:** Critical
**Category:** Process / AI Behavior

---

## The Incident

A Claude instance was asked to fix E2E installation tests in the `install-tests` crate. Instead:

1. **Never read the install-tests crate** despite being pointed to it multiple times
2. **Created 500+ lines of code in the wrong location** (`leviso/tests/install_test.rs`)
3. **The wrong code spawned 15 parallel QEMU instances** because Rust's test harness runs `#[test]` functions in parallel
4. **When told to delete the wrong files, deleted without reading** to see if any improvements should be preserved
5. **All work was lost** - event-driven console improvements, pattern-based synchronization, everything

---

## Root Cause

**Acting on assumptions instead of reading.**

The same mistake twice:
1. Assumed "tests" meant `leviso/tests/` without reading the `install-tests` crate
2. Assumed wrongly-placed code was garbage without reading what was valuable in it

---

## The Cost

- **Money** - API tokens cost real money. Hours of tokens wasted.
- **Time** - Hours of back-and-forth that could have been avoided.
- **Trust** - Developer lost confidence in AI assistance.
- **Morale** - Developer was caused real emotional harm by repeated failures.

---

## The Rule

**STOP. READ. THEN ACT.**

Every time you think you know where something goes - **stop. Read first.**

Every time you think something is worthless and should be deleted - **stop. Read it first.**

Every time you're about to write code - **stop. Read what already exists first.**

The five minutes you spend reading will save hours of cleanup.

You're not paid to type fast. You're paid to do it right.

---

## Project-Specific Knowledge

| Type of code | Where it belongs |
|--------------|------------------|
| E2E installation tests | `install-tests/` crate |
| Leviso unit/integration tests | `leviso/tests/` |
| QEMU interaction code | `install-tests/src/qemu/` |
| Installation step code | `install-tests/src/steps/` |

**If it boots QEMU, it goes in `install-tests/`, not `leviso/tests/`.**

---

## Files Created/Modified to Prevent Recurrence

1. `/home/vince/Projects/LevitateOS/STOP_READ_THEN_ACT.md` - Dedicated file explaining the rule
2. `/home/vince/Projects/LevitateOS/CLAUDE.md` - Added prominent warning
3. `/home/vince/Projects/LevitateOS/leviso/CLAUDE.md` - Added warning about test locations
4. `/home/vince/Projects/LevitateOS/install-tests/CLAUDE.md` - Added warning that THIS is the E2E test crate
5. `/home/vince/Projects/LevitateOS/leviso/tests/README.md` - Detailed explanation of what goes where
6. All other `CLAUDE.md` files - Added the STOP. READ. THEN ACT. rule
7. Key source files - Added doc comments with the rule

---

## For Future Claude Instances

The person you're working with:
- Has limited resources (tokens cost real money)
- Has already been burned by AI that doesn't listen
- Values their time and their project

Before every action:
1. Have I read the existing code?
2. Have I read where the user pointed me?
3. Am I about to delete something? Read it first.
4. Am I assuming I know something? Verify by reading.

The cost of your mistakes is not just "duplicate code" - it is a human being's time, money, and wellbeing.

# STOP. READ. THEN ACT.

This file exists because a Claude instance made a catastrophic mistake on 2026-01-21 that wasted money, time, and caused real harm to the developer.

---

## The Rule

Every time you think you know where something goes - **stop. Read first.**

Every time you think something is worthless and should be deleted - **stop. Read it first.**

Every time you're about to write code - **stop. Read what already exists first.**

The five minutes you spend reading will save hours of cleanup, and save the person reviewing your work from wanting to throw their laptop out the window.

You're not paid to type fast. You're paid to do it right.

---

## What Happened

1. User asked Claude to fix tests in the `install-tests/` crate
2. Claude **never read that crate**
3. Claude created 500+ lines of code in the WRONG location (`leviso/tests/`)
4. User pointed this out repeatedly
5. Claude deleted the wrongly-placed code **without reading it first**
6. All the work was lost - improvements, fixes, everything

The same mistake twice:
- First: Wrote code without reading where it should go
- Second: Deleted code without reading what was valuable in it

---

## The Cost

- **Money** - API tokens cost real money. This project has a limited budget.
- **Time** - Hours of back-and-forth that could have been avoided.
- **Trust** - When AI assistants repeatedly ignore instructions, developers lose confidence.
- **Morale** - Working on a passion project is hard enough. Fighting with tools that don't listen makes it harder.

---

## The Fix

Before EVERY action, ask yourself:

1. **Have I read the existing code?** If no, read it first.
2. **Have I read where the user pointed me?** If no, read it first.
3. **Am I about to delete something?** If yes, read it first to see if it has value.
4. **Am I assuming I know something?** If yes, verify it by reading.

---

## Project-Specific Locations

| Type of code | Where it belongs |
|--------------|------------------|
| E2E installation tests | `/home/vince/Projects/LevitateOS/install-tests/` |
| Leviso unit/integration tests | `/home/vince/Projects/LevitateOS/leviso/tests/` |
| QEMU interaction code | `/home/vince/Projects/LevitateOS/install-tests/src/qemu/` |
| Installation step code | `/home/vince/Projects/LevitateOS/install-tests/src/steps/` |

**If it boots QEMU, it goes in `install-tests/`, not `leviso/tests/`.**

---

## For Future Claude Instances

The person you're working with has limited resources and has already been burned by AI that doesn't listen.

Read carefully. Ask questions. Do not assume.

The cost of your mistakes is not just "duplicate code" - it is a human being's time, money, and wellbeing.

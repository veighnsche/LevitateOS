# Team 211 - Investigate cat Data Abort

## Team Members
- Cascade (Team 211)

## Task
Investigate why `cat --help` causes a Data Abort (0x24) at `0x104e8` with ESR `0x9200004f`.

## Investigation Logs
- Initial report: `cat --help` fails with Data Abort.
- ESR `0x9200004f` analysis:
    - EC: `0x24` (Data Abort from a lower Exception level)
    - ISS: `0x000004f`
    - DFSC: `0b001111` (Permission fault, level 3) or similar depending on the specific bits.
    - WnR: bit 6 is 1 (Write) -> It was a write operation.

## Plan
1. Find the `cat` binary in the workspace (likely in `initrd_root` or `userspace/levbox`).
2. Disassemble `cat` to see what's at `0x104e8`.
3. Check kernel page fault/data abort handler.

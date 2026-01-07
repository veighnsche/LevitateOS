# TEAM_147 Questions: TTBR0 Restoration Feature

## Q1: switch_to behavior during syscall yield

**Context:** When task2 yields to task1 during a syscall, task2's saved `ttbr0_el1` in the frame still points to task2's page tables.

**Question:** Should `switch_to()` update the saved TTBR0 in the yielding task's frame, or leave it unchanged?

**Options:**
- A) Leave unchanged — frame captures state at syscall entry (recommended)
- B) Update the frame — more complex, unclear benefit

**My Recommendation:** Option A

USER: A

---

## Q2: Task termination during syscall yield

**Context:** If task2 calls sys_read, yields, and is killed before resuming, what happens?

**Question:** Should we handle task termination during syscall yield?

**Options:**
- A) Don't handle — killing mid-syscall is undefined behavior for now (recommended)
- B) Add cleanup logic

**My Recommendation:** Option A for initial implementation

USER: A

---

## Q3: IRQ handler TTBR0 handling

**Context:** Timer IRQ from userspace could theoretically trigger schedule().

**Question:** Should IRQ handlers also save/restore TTBR0?

**Options:**
- A) Not initially — keep IRQ handlers simple (recommended)
- B) Yes — full parity with syscalls

**My Recommendation:** Option A — IRQ handlers should not yield

USER: A

# TEAM_067 — Multitasking & Scheduler (Phase 7)

**Created:** 2026-01-04
**Status:** PLANNING 🚧
**Role:** Feature Lead
**Team ID:** 067

---

## Objective

Kickstart Phase 7 of the LevitateOS Roadmap: **Multitasking & Scheduler**.
The immediate priority is implementing `unmap_page()` to support virtual memory reclamation, followed by context switching and the core scheduler.

---

## 📅 Roadmap (Phase 7)

- [ ] **Task 7.1: Virtual Memory Reclamation**
    - Implement `unmap_page()` in `mmu.rs`.
    - Handle TLB invalidation.
    - Integration with `PageAllocator` to free page table frames.
- [ ] **Task 7.2: Task Primitives**
    - Define `Task` struct and `TaskControlBlock` (TCB).
    - Implement task states (Ready, Running, Blocked).
- [ ] **Task 7.3: Context Switching**
    - Assembly implementation of `cpu_switch_to`.
    - Stack management for tasks.
- [ ] **Task 7.4: Scheduler**
    - Implement a basic Round-Robin scheduler.
    - Timer-based preemptive scheduling.

---

## 📝 Logs

| Date | Action | Note |
|------|--------|------|
| 2026-01-04 | Team registered. | Phase 7 planning initiated. |

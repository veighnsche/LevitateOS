# TEAM_147: Feature - TTBR0 Restoration for Syscall Context Switching

## Status: PLANNING

## Summary

Implement proper TTBR0 save/restore in the syscall exception path to enable preemptive multitasking during blocking syscalls.

## Problem Statement

Currently, `yield_now()` cannot be called from syscall handlers because the `eret` return path doesn't restore TTBR0. When a syscall yields to another task, the return happens with the wrong page tables, causing crashes.

## Goal

Enable blocking syscalls (like `sys_read`) to yield to other tasks while waiting for I/O, with proper address space restoration on return.

## Planning Files

- `docs/planning/ttbr0-restoration/phase-1.md` - Discovery
- `docs/planning/ttbr0-restoration/phase-2.md` - Design
- `docs/planning/ttbr0-restoration/phase-3.md` - Implementation
- `docs/planning/ttbr0-restoration/phase-4.md` - Integration & Testing

## Related

- TEAM_145: Identified this issue while investigating shell crash
- TEAM_143: Performance optimization that exposed the bug

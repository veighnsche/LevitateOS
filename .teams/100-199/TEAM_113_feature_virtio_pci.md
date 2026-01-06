# TEAM_113: VirtIO PCI Implementation

**Feature:** VirtIO PCI Transport Support
**Goal:** Enable VirtIO PCI support to fix GPU display cache coherency issues on AArch64.
**Previous Team:** TEAM_112 (Identified MMIO coherency issue)

## Context
TEAM_112 confirmed that `virtio-gpu-device` (MMIO) on AArch64/QEMU suffers from cache coherency issues for framebuffers, as the Guest maps them as Normal-Cacheable while the Host accesses them via a non-snooping path.
The standard solution is to use `virtio-gpu-pci`, where the AArch64 PCI controller nodes in the DTB/ACPI ensure correct memory attributes (Device/Uncached) for BAR mappings.

## Objectives
1.  Implement PCI ECAM scanning (Simple enumeration).
2.  Add `PciTransport` support to `levitate-virtio`.
3.  Migrate `virtio-gpu` initialization to use PCI transport.
4.  Verify display output.

## Planning
- Location: `docs/planning/virtio-pci/`
- Reference: `docs/handoffs/TEAM_112_gpu_display_analysis.md`

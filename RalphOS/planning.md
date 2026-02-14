# RalphOS Planning Notes

RalphOS is an "agents only" OS variant intended to host and supervise an opera of agents running a headless `ralphd` control plane plus isolated execution sandboxes.

These notes capture current decisions and the rough shape of the system. They are not a final spec.

## Goals

- Agents-only runtime: no desktop, no end-user interactive shell as the normal mode.
- Security-tight by default: minimize blast radius of any single agent / tool / "skill".
- Immutable base OS with atomic updates and easy rollback.
- Targets: `x86_64` and `aarch64`.
- A single northbound API for a companion Tauri desktop/mobile client to connect to.

## Non-Goals

- No chat-app gateway/connectors (WhatsApp/Telegram/etc.).
- RalphOS will not ask for, store, or manage user-provided external API keys.
  - If a tool requires authentication (e.g. Claude Code / Codex), the operator uses the tool's own login flow (`claude --login`, `codex --login`).
  - RalphOS only provides the environment/storage model for those credentials under strict policy.

## "Based On LevitateOS" Meaning

RalphOS being "based on LevitateOS" does not mean "same runtime contract".

- LevitateOS provides the build engine, kernel/userspace recipes, artifact pipeline, and checkpoint harness integration.
- RalphOS is a *profile/variant* that changes the runtime contract:
  - Immutable root
  - State strictly under `/var` (or an explicit data partition)
  - Agent workloads executed in sandboxes (not on the host)

## Immutable Base: A/B Slots (Chosen)

RalphOS uses an A/B update scheme:

- Disk has `SYSTEM_A` and `SYSTEM_B` partitions (system slots).
- Only one slot is active/booted at a time.
- Updates write a full new system image into the *inactive* slot.
- Boot selection flips to the new slot on reboot.
- A first-boot health check must mark the new slot as "good"; otherwise the bootloader falls back automatically.

RalphOS must support **A/B-only** deployment (immutability is not an optional mode for RalphOS).

### Why A/B

- Atomic updates: no partial package state.
- Rollback is cheap and reliable.
- Makes "immutable base" practical without requiring a specific CoW filesystem.

## Proposed Partition Layout (UEFI)

This layout works for both `x86_64` and `aarch64` UEFI boots:

- `EFI` (FAT32): bootloader + boot entries for both A and B slots
- `SYSTEM_A` (ext4 or image): immutable root for slot A
- `SYSTEM_B` (ext4 or image): immutable root for slot B
- `VAR` (ext4): writable state
  - `/var/lib/ralphd` (control plane state, policy, queues)
  - `/var/log` (audit logs, system logs)
  - `/var/lib/ralphd/sandboxes` (images, overlays, snapshots)
- Optional `RECOVERY`: minimal rescue environment

## Boot + Rollback Mechanics

Key idea: the bootloader needs a "try/commit" mechanism.

- When updating to the inactive slot, we set "try boot slot X for N attempts".
- On successful boot, a health-check service commits the slot (clears the try counter).
- If the slot is not committed after N attempts, bootloader falls back to the previous slot.

### Health Check ("Commit Slot")

The system should only commit the new slot after it proves the minimum viable system is working.
For RalphOS, that likely means:

- `ralphd` starts successfully.
- The policy engine loads.
- A sandbox can be created and a trivial command can run inside it.
- (Optional) Northbound API is reachable on localhost.

## Sandboxing (Security Boundary)

RalphOS is for agents; therefore sandboxing is the core security feature.

Baseline expectations:

- Default: agents do not run directly on the host.
- Each job (or each agent) runs in an isolated sandbox with:
  - filesystem separation (read-only base + per-job writable overlay)
  - resource quotas (CPU, RAM, disk, pids, open files)
  - explicit network policy (default deny egress)
  - explicit mounts only (no host filesystem passthrough)

Implementation choice is still open:

- Prefer VM/microVM (KVM) as the primary boundary.
- Containers can be used inside the VM boundary for convenience, but should not be the only wall in "prod".

## Credentials Policy (No User API Keys)

- RalphOS does not accept arbitrary API keys from users.
- Authentication to external tools happens through the tool's native login flows.
- Credentials are stored under a dedicated, root-owned location and exposed to sandboxes only via policy.
- Long-lived secrets should be minimized; prefer short-lived tokens where possible.

Open question: whether credentials are per-agent, per-tenant, or global-to-host.

## Networking

- Host exposes a single northbound API for the companion client (your Tauri desktop/mobile app).
- Sandboxes run in isolated networks.
- Default-deny egress for sandboxes; allowlist by policy (destinations, ports, protocols).
- Full egress logging for auditing/debugging.

## Auditability

RalphOS needs an audit trail suitable for "run a company" automation:

- tool invocations and their parameters (redacted where needed)
- process spawns
- file writes (paths, hashes)
- outbound network destinations
- secret reads (what scope, which agent/job)
- admin actions (updates, policy changes, break-glass)

## Checkpoints (How This Affects Our Repo)

RalphOS will show up in the checkpoint tables for:

- `x86_64`: LevitateOS, RalphOS, AcornOS, IuppiterOS
- `aarch64`: LevitateOS, RalphOS, AcornOS

RalphOS-specific "definition of done" will likely differ at CP3+ because the OS is not user-interactive and may not support the same "installed desktop" flows.

## Open Questions

- Bootloader choice: `systemd-boot` vs GRUB for A/B with try/commit semantics.
- Storage format for slot images: ext4 root partitions vs monolithic images vs dm-verity.
- Sandbox boundary for v0: microVM (KVM) vs containers.
- Multi-tenant scope: single-tenant host with many agents vs multi-tenant (multiple companies) on one host.
- Remote admin: SSH/WireGuard allowed at all, or strictly local console with break-glass.

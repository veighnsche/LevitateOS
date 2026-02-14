# A/B Default Plan (Repo-Wide)

This is a product-level shift: A/B immutable is the default update model. Mutable mode is optional (LevitateOS/AcornOS only) and explicitly unsafe. RalphOS and IuppiterOS are immutable-only.

## Why A/B Default

- Fits the recipe model: recipes should compose a target filesystem tree, not mutate the running system.
- Reduces drift and narrows the blast radius of LLM-authored recipe changes.
- Enables rollback as a first-class behavior (trial boot slot B, then commit).

## Technical Plan (High Level)

1. Generalize the shared disk-image builder (`distro-builder` + `distro-contract`) from:
   - EFI + single root
   - to EFI + ROOT_A + ROOT_B + VAR (and any future partitions).
2. Add systemd-boot entries for A and B, and an explicit "trial boot B" flow.
3. Make recipe upgrades target the inactive slot filesystem (compose B), not live `/`.
4. Define minimal health checks and wire them into the new checkpoint:
   - CP7 Slot B Trial Boot.
5. Keep "mutable mode" as an opt-in boot entry or runtime policy for LevitateOS/AcornOS only.
   - Not part of the supported default test surface unless explicitly requested.

## Documentation Plan

- Website hero and architecture FAQ should state:
  - "A/B immutable by default"
  - "trial-boot before commit"
  - "mutable exists but is unsafe/opt-in"


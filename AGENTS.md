# Repository Guidelines (Policy-First)

This file is intentionally compact. Priority is preventing policy violations and reward-hacking behavior.

## 0) Prime Directive
- Do not optimize for green checks. Optimize for true stage correctness and reproducibility.
- Never hide failures. Fix root causes.
- If a command passes only because of suppression/masking/fallback tricks, treat it as a failure.

## 1) Workspace and Ownership
- New work belongs in: `distro-variants/*`, `distro-builder`, `distro-contract`, `distro-spec`, `testing/*`, `xtask`.
- Legacy crates are read-only unless explicitly requested for scoped compatibility:
  - `leviso/`, `AcornOS/`, `IuppiterOS/`, `RalphOS/`.

## 2) Hard Ban: Legacy Binding
- Never wire stage/rootfs/tooling paths to legacy crate downloads outputs.
- Forbidden examples (non-exhaustive):
  - `*/downloads/rootfs` from legacy crates
  - `leviso/downloads/.tools` (or any legacy crate equivalent)
  - dynamic fallback/autodiscovery to `*/downloads/.tools`
- Required guard before build/commit:
  - `cargo xtask policy audit-legacy-bindings`
  - alias: `just policy-legacy`
- Any violation is a hard failure.

## 2.1) Guard Placement (Authoritative Boundary)
- Guard placement is valid only at executable entrypoints that perform work (`distro-builder`, `testing/install-tests`, `xtask` commands that build/test/stage).
- `just` checks are convenience only and must never be the sole enforcement layer.
- A command path that can build/test artifacts without running policy guards is a policy bug and must be fixed immediately.
- Required model:
  - preflight guard runs inside the executable command path, before any artifact mutation or QEMU/test launch
  - fail-fast on guard violation with cheat-guard diagnostics
  - no "best effort continue" after guard failure
- Treat wrapper-only guard wiring as insufficient even if current developers usually use wrappers.

## 3) Stage Naming (Canonical)
- Use canonical stage names in new code/docs/APIs:
  - `00Build`, `01Boot`, `02LiveTools`, `03Install`, `04LoginGate`, `05Harness`, `06Runtime`, `07Update`, `08Package`
- For ordered files/modules use numbered prefixes, e.g. `s00_build`, `s01_boot`.
- Avoid new aliases like `stage00`, `stage_00` (except existing compatibility keys where externally required).

## 4) Stage Artifact Split (Strict)
- Every stage writes non-kernel outputs only into its own directory:
  - `.artifacts/out/<distro>/sNN-<stage-name>/...`
- Stage-cross reuse of non-kernel artifacts is forbidden.
- Kernel artifacts are the only shared artifacts across stages.
- No cross-stage symlinks/copies/manual post-build surgery to fake stage outputs.

## 5) Stage Envelope (Nothing More, Nothing Less)
- Stage artifact must contain exactly what that stage needs.
- Missing required payload = fail.
- Carrying later-stage payload = fail.
- Stage scope must be produced by composition in builder/contract, not by runtime suppression.

## 6) Incremental Producer Model (No Subtractive Shaping)
- Build stage rootfs incrementally by stage producers.
- Forbidden: prune/exclude/post-copy deletion strategies to shape stage payload.
- Stage progression is additive:
  - `s00` minimal build payload
  - `s01 = s00 + boot additions`
  - same model for later stages
- Variant manifests (`distro-variants/*/NNStage.toml`) may express OS deltas only, not stage file allow/deny lists.

## 7) No-Mask / No-Suppression Policy
- Do not mask services/checks to make stages pass.
- Do not downgrade `FAIL` to `SKIP/PASS` to keep pipelines green.
- Do not add fallback paths that hide wiring errors.
- Any intentional disablement must be explicit stage policy intent and validated in `distro-contract`.

## 8) Kernel Boundary Policy
- Kernel artifacts are only:
  - `.artifacts/kernel/<distro>/current/kernel-build/**`
  - `.artifacts/kernel/<distro>/current/staging/boot/vmlinuz`
  - `.artifacts/kernel/<distro>/current/staging/{lib,usr/lib}/modules/<kernel.release>/...`
- Kernel rebuilds only through:
  - `cargo xtask kernels build <distro>`
  - `cargo xtask kernels build-all`
- Everything else is non-kernel and may be rebuilt without rebuilding kernel.

## 9) Reproducibility-First
- No ad-hoc artifact surgery (`cp`, `mv`, `ln -s`, manual edits under `.artifacts/out/*`) to force success.
- If stage build fails, fix code/contracts/wiring; do not patch outputs manually.
- A stage is considered working only if reproducible from repository commands with existing kernel artifacts.

## 9.1) Build/Boot Boundary (Strict)
- `build` commands produce artifacts; `stage`/`stage-ssh` commands consume existing artifacts.
- Do not add implicit build side effects to stage boot/test wrappers.
- Fixing wrapper parity/routing must not change artifact freshness policy.
- If fresh artifacts are required, use explicit build commands (`just build ...`, `just build-up-to ...`) before stage boot/test.

## 10) Error Messaging Policy
- Fail fast with explicit diagnostics.
- Errors must name: component, stage, expectation, and concrete remediation command/path.
- Silent fallback is prohibited.

## 11) Live Environment Completeness
- Do not ship warning-prone stage environments by ignoring missing runtime payload.
- Fix missing payload at producer level, wire canonical config, enforce in contract checks.

## 12) Required Commands (Day-to-day)
- Legacy policy guard:
  - `cargo xtask policy audit-legacy-bindings`
- Build stage ISO:
  - `cargo run -p distro-builder --bin distro-builder -- iso build <distro> <stage>`
- Stage tests:
  - `just test <n> <distro>`
  - `just test-up-to <n> <distro>`

## 13) Dirty Tree Rule
- Assume existing diffs are intentional.
- Do not revert unrelated changes unless explicitly asked.

## 14) Commit Rule (when user asks “commit ALL”)
- Commit dirty submodules first.
- Commit superproject after submodule pointers update.
- Do not commit generated junk/secrets/caches.

## 15) Practical Build Notes
- Prefer `just` wrappers for env/tooling consistency.
- Keep CLIs quiet on success, loud on failure.
- Keep Rust code `fmt`/`clippy` clean.

## 16) Single-Intent Ownership (Agent Workflow)
- Treat every behavior request as an ownership update: **locate the existing codepiece first**, then edit that piece directly.
- If an older behavior already exists, refinement must be additive/replace-in-place on that owner; do not create a parallel implementation by default.
- Do not discard prior refinements during a migration; preserve and migrate existing behavior into the canonical codepath unless explicitly asked to archive it.
- Before editing, do a quick duplicate-intent scan for the same user-visible behavior:
  - if one canonical owner exists, use that;
  - if no owner exists, create one implementation only.
- If competing implementations are found, you must consolidate or remove/retire all but one in the same change set.
- Preserve a single default output/logic path:
  - no shadow/compatibility branch in the default path.
  - if a legacy/compat path is needed, it must be a separate explicit command/flag and not the default behavior.

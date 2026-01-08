# TEAM_291: AArch64 Cross-Compilation Investigation

## Phase 1: Understand the Symptom
The user noted that AArch64 builds require "cross stuff". On a standard GitHub runner (Ubuntu `x86_64`), building for `aarch64-unknown-none` requires:
1.  **Rust Toolchain Targets:** `aarch64-unknown-none`.
2.  **Cross-Linker:** A linker capable of emitting AArch64 ELF files.
3.  **Cross-Objcopy:** A tool to convert ELF to raw binary for AArch64 boot protocols.

## Phase 2: Form Hypotheses
1.  **Missing Cross-Linker:** The `aarch64-unknown-none` target in Rust often defaults to the system linker, which on Ubuntu is `ld` (x86_64). Without specifying a cross-linker (like `aarch64-linux-gnu-ld` or `lld`), the build will fail during linking.
2.  **Rust Linker Configuration:** The `.cargo/config.toml` only specifies `-Tlinker.ld` but doesn't set a `linker = "..."` for AArch64.
3.  **Objcopy Presence:** `xtask` explicitly calls `aarch64-linux-gnu-objcopy`. The workflow installs `gcc-aarch64-linux-gnu`, which provides this. This part is likely okay.

## Phase 3: Evidence Gathering
- **Linker Configuration:** `@/home/vince/Projects/LevitateOS/.cargo/config.toml:1-4` shows no specific linker defined for `aarch64-unknown-none`.
- **Workflow Dependencies:** `@/home/vince/Projects/LevitateOS/.github/workflows/release.yml:56` installs `gcc-aarch64-linux-gnu`.
- **Local Build Success:** My local build succeeded, but I may have `lld` or `aarch64-linux-gnu-gcc` installed and configured as a default or in my environment.

## Phase 4: Narrow Down Root Cause
The root cause is likely the **lack of an explicit cross-linker configuration** in `.cargo/config.toml`. While `gcc-aarch64-linux-gnu` is installed in the workflow, Rust doesn't automatically know to use `aarch64-linux-gnu-gcc` or `aarch64-linux-gnu-ld` as the linker unless specified.

## Phase 5: Decision
The fix is small and high confidence. I will update `.cargo/config.toml` to specify the cross-linker for the AArch64 target.

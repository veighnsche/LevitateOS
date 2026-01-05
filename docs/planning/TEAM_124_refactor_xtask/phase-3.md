# Phase 3: Migration

## Migration Strategy
Since we are modifying the binary interface of a development tool (`xtask`), we can be slightly more aggressive than with a public library API.
However, scripts like `run.sh` depend on `xtask`.

1. **Implement new structure** in `xtask` (Phase 2).
2. **Update `run.sh`** to use new commands (e.g. `cargo xtask build all` or just `cargo xtask build`).
3. **Verify** CI/CD or other scripts (none known besides `run.sh`).

## Step 1: Migrate `run.sh`
`run.sh` currently calls:
- `cargo xtask build`

We need to ensure `cargo xtask build` still works (defaults to `All`?) or update `run.sh`.
Our plan in Phase 2 says `Build` is a subcommand group.
If we make `Build` an enum variant with arguments, `clap` requires a subcommand.
To support `cargo xtask build` (no subcommand), we might need `#[command(subcommand)]` on an Option?
Or just mandate `cargo xtask build all`?
**Decision:** Mandate `cargo xtask build all` or `kernel` for clarity. Update `run.sh`.

## Step 2: Update Documentation
Update `README.md` or `xtask` help text to reflect new hierarchy.

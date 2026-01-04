# Initramfs - Phase 5: Polish & Handoff

## 1. Cleanup
- [ ] Remove `scripts/make_initramfs.sh` once integrated into the main build script if needed.
- [ ] Clean up debug logs in `fdt` and `cpio` modules.

## 2. Documentation
- [ ] Add documentation comment to `Initramfs` struct.
- [ ] Document the intended use of initramfs for Phase 8.

## 3. Handoff
- [ ] Update `docs/ROADMAP.md` to mark Initramfs as complete.
- [ ] Document any limitations (e.g., maximum size supported by initial mapping).

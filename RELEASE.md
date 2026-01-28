# LevitateOS Release Procedure

This document describes how to create and publish a LevitateOS release.

## Release Types

- **Tech Preview**: Early access release, incomplete features, experimental. Tag: `v0.X.0-preview`
- **Alpha**: Core features working, recipe system documented, documented gaps. Tag: `v0.X.0-alpha`
- **Beta**: Feature-complete for stated scope, minimal known issues. Tag: `v0.X.0-beta`
- **Stable**: Production-ready (if that ever happens). Tag: `v1.X.0`

## Pre-Release Checklist

- [ ] All tests passing: `cd testing/install-tests && cargo test -- --nocapture`
- [ ] ISO builds without errors: `cd leviso && cargo run -- build`
- [ ] Installation guide tested end-to-end
- [ ] Known issues documented (see below)
- [ ] Release notes drafted
- [ ] No uncommitted changes in git

## Release Procedure

### Step 1: Build ISO

```bash
cd /home/vince/Projects/LevitateOS/leviso
cargo run -- build
```

Output: `/home/vince/Projects/LevitateOS/output/levitateos.iso`

### Step 2: Generate Checksums

```bash
cd /home/vince/Projects/LevitateOS/output

# SHA256
sha256sum levitateos.iso > SHA256SUMS

# SHA512 (optional but recommended)
sha512sum levitateos.iso > SHA512SUMS

# Verify
sha256sum -c SHA256SUMS
# Expected output: levitateos.iso: OK
```

### Step 3: Collect Release Metadata

#### Known Issues Template

List blockers from this release:
```markdown
## Known Issues

- **No desktop environment pre-installed** - Install your own (Sway, GNOME, KDE, etc.)
- **Limited recipe collection** - Most software requires writing a recipe first
- **Documentation incomplete** - Help welcome at https://github.com/LevitateOS/LevitateOS/issues
```

#### Checksums for Release Notes

```bash
cat /home/vince/Projects/LevitateOS/output/SHA256SUMS
cat /home/vince/Projects/LevitateOS/output/SHA512SUMS
```

### Step 4: Create GitHub Release

Use GitHub CLI:

```bash
cd /home/vince/Projects/LevitateOS

gh release create v0.1.0-preview \
  --title "LevitateOS Tech Preview 0.1.0" \
  --draft \
  --notes "$(cat <<'EOF'
# LevitateOS Tech Preview 0.1.0

⚠️ **WARNING: This is a tech preview release. Not for daily use.**

LevitateOS is a Linux distribution for power users who want to build their own system from the ground up, Arch-style. This release demonstrates core functionality but lacks many features needed for daily driving.

## What Works

- ✅ UEFI and BIOS boot
- ✅ Manual Arch-style installation via `recstrap`
- ✅ Filesystem generation with `recfstab`
- ✅ Chroot helper with `recchroot`
- ✅ Package manager `recipe` with Rocky/Alpine extraction
- ✅ 86+ base packages available

## What Doesn't Work

- ❌ No desktop environment pre-installed
- ❌ Limited package recipes (you write what you need)
- ❌ Documentation incomplete
- ❌ No graphical installer

## Hardware Requirements

- **CPU:** x86-64-v3 (Haswell 2013+, Ryzen 1000+)
- **RAM:** 8 GB minimum
- **Storage:** 64 GB NVMe
- **Firmware:** UEFI or BIOS

## Installation

1. Download ISO and verify SHA256 below
2. Create bootable USB: `dd if=levitateos.iso of=/dev/sdX bs=4M`
3. Boot and follow installation guide (see docs link below)

## Documentation

- 📖 [Installation Guide](https://levitateos.org/docs/installation)
- 🆘 [Troubleshooting](https://levitateos.org/docs/troubleshooting)
- 📝 [Known Issues](https://levitateos.org/docs/known-issues)

## Support

- 💬 [GitHub Discussions](https://github.com/LevitateOS/LevitateOS/discussions)
- 🐛 [Bug Tracker](https://github.com/LevitateOS/LevitateOS/issues)
- 📧 [Contact](https://levitateos.org/about)

## Verification

### SHA256
\`\`\`
$(cat output/SHA256SUMS)
\`\`\`

### SHA512
\`\`\`
$(cat output/SHA512SUMS)
\`\`\`

Verify with:
\`\`\`bash
sha256sum -c SHA256SUMS
\`\`\`

## Known Limitations

- No graphical installer (manual CLI installation only)
- No pre-configured desktop (you choose: Sway, GNOME, KDE, Xfce, etc.)
- Limited recipe ecosystem (write your own or use Rocky/Alpine packages)
- This is a nights-and-weekends project—expect rough edges

## Contributing

New to LevitateOS? Check out [CONTRIBUTING.md](https://github.com/LevitateOS/LevitateOS/blob/master/CONTRIBUTING.md)

Questions? Start a [discussion](https://github.com/LevitateOS/LevitateOS/discussions).

---

**Release Date:** $(date -I)
**Commit:** $(git rev-parse --short HEAD)
EOF
)" \
  output/levitateos.iso \
  output/SHA256SUMS \
  output/SHA512SUMS
```

**If not using CLI:**
- Go to https://github.com/LevitateOS/LevitateOS/releases/new
- Tag: `v0.1.0-preview`
- Title: `LevitateOS Tech Preview 0.1.0`
- Description: Use template above
- Attach: `levitateos.iso`, `SHA256SUMS`, `SHA512SUMS`
- Select: **"Set as pre-release"** checkbox
- Click: **"Publish release"**

### Step 5: Update Download Page

Edit `docs/website/src/routes/download/+page.svelte`:

```svelte
<h2>Download LevitateOS {version}</h2>

<div class="release-info">
  <p class="tech-preview-warning">
    ⚠️ Tech Preview: Not for daily use. For early adopters and distro enthusiasts.
  </p>

  <div class="download-button">
    <a href="https://github.com/LevitateOS/LevitateOS/releases/download/v0.1.0-preview/levitateos.iso">
      Download ISO (levitateos-0.1.0-preview.iso)
    </a>
  </div>

  <div class="checksums">
    <h3>Verify Download</h3>
    <code>SHA256: {sha256}</code>
    <p>Full checksums: <a href="https://github.com/LevitateOS/LevitateOS/releases/download/v0.1.0-preview/SHA256SUMS">SHA256SUMS</a></p>
  </div>

  <div class="release-notes">
    <a href="https://github.com/LevitateOS/LevitateOS/releases/tag/v0.1.0-preview">
      Full release notes →
    </a>
  </div>
</div>
```

### Step 6: Verify Release

1. Visit: https://github.com/LevitateOS/LevitateOS/releases/tag/v0.1.0-preview
2. Download ISO, verify checksum matches
3. Test boot in QEMU or physical hardware
4. Verify download page displays correctly

## Post-Release

- [ ] Announce on /r/linux (follow subreddit rules)
- [ ] Post to Hacker News (optional, "Show HN" format)
- [ ] Update README.md with Tech Preview badge
- [ ] Create GitHub Discussions categories if not already done
- [ ] Monitor for immediate issues and respond quickly
- [ ] Document bugs/feedback for next release

## Subsequent Releases

For v0.2.0-preview, v0.3.0-alpha, etc.:

1. Update version numbers in commands above
2. Increment release number
3. Update known issues based on feedback
4. Rebuild ISO (`cargo run -- build`)
5. Regenerate checksums
6. Follow Steps 4-6 above

## Release Cadence Recommendation

- **Tech Preview:** Every 1-2 weeks (rapid iteration based on feedback)
- **Alpha:** Every 3-4 weeks (let community test, collect recipes)
- **Beta:** Every 4-6 weeks (polish, fix regressions)
- **Stable:** As needed (bug fixes only)

## Emergency Hotfix Release

If critical bug discovered after release:

```bash
# Fix in code
git commit -m "fix: critical issue X"

# Tag hotfix
git tag v0.1.1-preview

# Build and release using steps above, with tag v0.1.1-preview
```

## Rollback

If release proves unusable:

```bash
# Delete local tag
git tag -d v0.1.0-preview

# Delete remote tag
git push origin :refs/tags/v0.1.0-preview

# Delete GitHub release via web UI
# (Visit releases page, click delete)
```

## Checklist for Release Day

- [ ] All tests pass
- [ ] ISO builds successfully
- [ ] Checksums generated and verified
- [ ] Release notes written
- [ ] GitHub release created with assets
- [ ] Download page updated
- [ ] Announcement drafted
- [ ] Announcement posted
- [ ] Monitor for issues first 48 hours

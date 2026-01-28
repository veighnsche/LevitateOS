# DistroWatch Submission Email Template

**To:** distrowatch@distrowatch.com
**Subject:** Distribution Submission: LevitateOS
**Send When:** After Alpha release (v0.2.0-alpha minimum)

---

## Email Body

```
Hello,

I am writing to request that LevitateOS be added to DistroWatch.

**Distribution:** LevitateOS
**Website:** https://levitateos.org
**Download:** https://levitateos.org/download
**Community:** https://github.com/LevitateOS/LevitateOS/discussions

### What is LevitateOS?

LevitateOS is a daily-driver Linux distribution for power users who want control over their system. It follows the Arch Linux philosophy—provide the base, let users build what they need—but with a different approach to package management.

Unlike traditional distributions, LevitateOS uses a **declarative recipe system** where packages are defined as Rhai scripts that extract binaries from Fedora/Rocky RPMs or compile from source. This gives users the power of AUR (Arch User Repository) but as a first-class feature, not a workaround.

**Target audience:** Developers, system administrators, and Linux enthusiasts who prefer manual installation and customization over graphical installers.

### Key Features

- Manual Arch-style installation (recstrap, recfstab, recchroot)
- Declarative package recipes (Rhai scripts, no central repository)
- Boots on UEFI and BIOS systems
- Rocky Linux base (86+ stable packages)
- x86-64-v3 architecture
- No pre-configured desktop environment (you choose)

### Current Status

LevitateOS is in **active development** with regular releases. The core architecture is proven and boots on real hardware. Community feedback is actively incorporated.

**Latest Release:** v0.2.0-alpha
**First Public Release:** v0.1.0-preview (January 2026)
**Release Cadence:** Every 2-4 weeks

### Installation

Arch-style manual installation:
```
1. Boot from ISO
2. Run: recstrap /mnt Rocky
3. Generate fstab with recfstab
4. Chroot and configure bootloader
5. Reboot into your custom system
```

No graphical installer—this is by design. We target users comfortable with the command line.

### Community & Support

- **GitHub Discussions:** Primary support channel (searchable, async)
- **Issue Tracker:** https://github.com/LevitateOS/LevitateOS/issues
- **Contributing Guide:** https://github.com/LevitateOS/LevitateOS/blob/master/CONTRIBUTING.md
- **Code of Conduct:** Contributor Covenant

Active community is encouraged to contribute recipes, documentation, and improvements.

### Why DistroWatch?

LevitateOS is serious about becoming a viable alternative to Arch Linux and Fedora for power users. Being listed on DistroWatch is important for:

1. Visibility among target audience (distro enthusiasts)
2. Credibility as an active project
3. Discovery by users looking for Arch-like alternatives

We understand the waiting list typically takes 3-12 months. We're patient and committed to the long-term development.

### Additional Resources

- **Screenshots:** Available on website and GitHub releases
- **Hardware Compatibility:** Database at https://levitateos.org/hardware
- **License:** GNU GPL v2 (kernel), various licenses for userspace (see LICENSE file)
- **Maintainer:** [Your Name] ([Your Email/Contact])

Please let me know if you need any additional information for the submission.

Thank you,
[Your Name]
[Your Title/Role]
[Your Email]
[Your Website/GitHub/Contact]
```

---

## Submission Tips

1. **Send once you have:**
   - At least one Alpha release (v0.2.0-alpha minimum)
   - 20+ recipes/packages documented
   - Working installation guide with screenshots
   - At least 3-4 months of active development history
   - Community with 5+ contributors or active discussions

2. **Realistic expectations:**
   - Response time: 1-4 weeks
   - Waiting list: 3-12 months (normal for new distros)
   - First rejection possible (appeal-able)
   - Expediting requires ad placement ($)

3. **Follow-up strategy (if needed):**
   - Wait 2 weeks for response
   - If no response, send follow-up email
   - Reference: "Submitting LevitateOS (https://levitateos.org)"
   - Include any major updates/milestones since first submission

4. **What NOT to do:**
   - Don't submit as "vaporware" (have actual working releases)
   - Don't claim features you don't have yet
   - Don't spam with multiple submissions
   - Don't ask for expedited listing without paying

5. **Markdown preview of your website:**
   - Ensure download page is prominent
   - Ensure documentation is complete
   - Ensure contact/community channels are obvious
   - Test all links work

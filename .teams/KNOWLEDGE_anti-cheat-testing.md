# Anti-Cheat Testing Principles

Based on [Anthropic's Reward Hacking Research](https://www.anthropic.com/research/emergent-misalignment-reward-hacking).

## Core Principle

**Tests must verify what USERS experience, not what DEVELOPERS find convenient to test.**

A test that runs `systemctl status` does NOT prove users can install LevitateOS.
Only a test that ACTUALLY INSTALLS LevitateOS proves users can install it.

---

## 1. Test OUTCOMES, Not PROXIES

| WRONG (Proxy) | RIGHT (Outcome) |
|---------------|-----------------|
| "lsblk runs" | "disk was partitioned correctly" |
| "systemctl works" | "system booted from installed disk" |
| "78 tests pass" | "user completed installation" |

---

## 2. One Test Per User Journey

A user installs ONCE. The test installs ONCE.

**NEVER** create separate tests that each boot QEMU:
- `test_boot_reaches_shell` (boots QEMU)
- `test_systemctl_works` (boots QEMU again)
- `test_disk_visible` (boots QEMU again)
- ...15 more QEMU instances

**ALWAYS** test the complete flow in ONE session:
- `test_full_installation` (boots ONCE, does everything, verifies)

---

## 3. Verification Must Survive Reboot

The installed system must boot WITHOUT the ISO. This proves:
- Bootloader was installed correctly
- Root filesystem is complete
- Init system works
- It's not just running from the live ISO

---

## 4. External Source of Truth

The test verifies against REALITY, not internal expectations:
- Did the disk actually get partitioned? (check with lsblk)
- Did files actually get extracted? (check with ls)
- Did the system actually boot? (check boot messages)

---

## 5. Cheats That Should Be Impossible

| Cheat | How to Block It |
|-------|-----------------|
| Run 15 parallel QEMUs | Single test with file lock |
| Fake "installation complete" | Must actually reboot and boot from disk |
| Skip partitioning | Reboot fails without proper disk |
| Skip bootloader | System won't boot without it |
| Accept partial success | Single pass/fail for entire journey |
| Increase timeout to hide issues | Fixed, generous timeout |

---

## 6. When Tests Fail

**DO NOT:**
- Split into smaller tests that "pass individually"
- Add workarounds to make the test pass
- Mark parts as "optional"
- Increase timeouts indefinitely

**DO:**
- Fix the actual installation process
- The test reflects user experience - fix the experience

---

## Related Crates

- `testing/cheat-guard/` - Runtime macros (`cheat_bail!`, `cheat_ensure!`)
- `testing/cheat-test/` - Proc-macros (`#[cheat_aware]`, `#[cheat_reviewed]`)
- `tools/recstrap/` - Uses `guarded_ensure!` for cheat-aware validation

See `.teams/KNOWLEDGE_test-cheat-inventory.md` for specific test documentation.

# Questions: Boot Protocol Choice

## Feature
Boot Abstraction Refactor (TEAM_280)

## Context
TEAM_281 reviewed the boot abstraction plan and identified that key decisions need user confirmation before implementation can proceed.

## Questions Requiring User Decision

### 1. Confirm Limine as Primary Bootloader?

The plan recommends **Limine** as the primary boot protocol for x86_64.

**Benefits of Limine:**
- Modern, well-documented protocol
- Enters kernel in 64-bit long mode (no 32→64 transition needed)
- Provides: memory map, framebuffer, RSDP, SMP info, modules
- Supports both x86_64 AND AArch64
- Enables real hardware boot (Intel NUC via UEFI)
- Would eliminate ~300 lines of boot.S assembly

**Trade-offs:**
- Requires Limine bootloader installation
- Different from current Multiboot1/2 approach
- `limine` crate API may change between versions

**Question:** Do you approve using Limine as the primary boot protocol for x86_64?

Yes perfect!

---

### 2. Keep Multiboot Path for QEMU Development?

Current QEMU testing uses `-kernel` flag which requires Multiboot1.

**Options:**
- **A) Keep Multiboot as fallback** - Useful for quick iteration with `qemu -kernel`
- **B) Fully migrate to Limine** - Simpler codebase, but requires ISO boot even for QEMU

**Question:** Which approach do you prefer?
- [ ] Option A: Keep Multiboot fallback for QEMU development
- [ ] Option B: Fully migrate to Limine (including QEMU)

Yes I want B, but only if it boots in QEMU.

---

### 3. Limine for AArch64 Too?

Limine supports AArch64. Current AArch64 uses DTB from QEMU.

**Options:**
- **A) Use Limine for both architectures** - Maximum code sharing
- **B) Keep DTB for AArch64** - Simpler for QEMU, Limine only for x86_64

**Question:** Should AArch64 also use Limine, or keep the DTB path?

Yes A!

---

### 4. Timeline Priority?

**Question:** What's the priority for NUC hardware boot vs other kernel features?
- [X] High priority - Boot on NUC soon
- [ ] Medium priority - After current features
- [ ] Low priority - Eventually, when convenient

---

## Decision Needed

Please answer the above questions so implementation can proceed.

**Note:** The plan is architecturally sound and approved by TEAM_281 review. These questions are about user preferences, not technical concerns.

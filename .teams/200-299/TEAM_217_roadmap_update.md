# Team 217: Roadmap Update (Userland & Storage Strategy)

## **Objective**
Update the project `ROADMAP.md` to reflect the prioritized strategy for Userland (std) support via Linux ABI compatibility and the decision to delay physical storage drivers (Ext4) in favor of VFS/tmpfs stability.

## **Context**
Based on the "Action Required" roadmap definition:
1. **Decision**: Aim for Linux Binary Compatibility (High ROI).
2. **Strategy**: Sprint on Linux Syscall Compatibility first.
3. **Storage**: Delay Ext4; focus on VFS abstraction and tmpfs for now.
4. **Target**: Standard Rust applications running via `std`.

## **Actions**
- [x] Update `ROADMAP.md` to prioritize Phase 15+ around Linux ABI. (TEAM_217)
- [x] Explicitly document the decision to delay Ext4 in the Storage Strategy section. (TEAM_217)
- [x] Align Part II (Userspace Expansion) with the "Linux ABI First" goal. (TEAM_217)
- [x] Document Linux ABI compatibility findings for future teams in `docs/specs/LINUX_ABI_GUIDE.md`. (TEAM_217)

## **Completion Status**
- `ROADMAP.md` updated with "Strategy: Linux ABI Compatibility" section.
- Phase 14 updated with storage strategy (delaying Ext4).
- Phase 15 updated with high priority for Linux ABI/Syscall compatibility.
- `LINUX_ABI_GUIDE.md` created with critical alignment knowledge (Stat, auxv, TLS).
- Linked guide in `ARCHITECTURE.md`.

## **Traceability**
- **Trigger**: User request for roadmap definition.
- **Reference**: `docs/ROADMAP.md`

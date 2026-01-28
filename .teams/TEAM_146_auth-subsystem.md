# TEAM_146: Login/Authentication Subsystem Consolidation

## Objective
Consolidate scattered login/authentication code into a dedicated subsystem in `distro-spec/src/shared/auth/`, document all requirements comprehensively, and fix the root password issue on installed systems.

## Current Status
- [x] Phase 1: Requirements documentation (`distro-spec/src/shared/auth/requirements.md`)
- [x] Phase 2: Auth subsystem structure created
  - [x] `distro-spec/src/shared/auth/mod.rs`
  - [x] `distro-spec/src/shared/auth/components.rs`
  - [x] `distro-spec/src/shared/auth/pam.rs`
  - [x] `distro-spec/src/shared/auth/getty.rs`
  - [x] `distro-spec/src/shared/auth/ssh.rs`
  - [x] Updated `distro-spec/src/shared/mod.rs` to include auth module
- [x] Phase 3: Extract constants from components.rs and add re-exports
  - [x] Moved AUTH_BIN, AUTH_SBIN, SHADOW_SBIN to auth/components.rs
  - [x] Moved SSH_BIN, SSH_SBIN to auth/components.rs
  - [x] Moved PAM_MODULES, PAM_CONFIGS, SECURITY_FILES to auth/components.rs
  - [x] Moved SUDO_LIBS to auth/components.rs
  - [x] Added re-exports in distro-spec/src/shared/components.rs
  - [x] Verified cargo check passes (no errors, backwards compatible)
- [ ] Phase 4: Update imports in leviso (if needed - existing code uses distro_spec::shared)
- [ ] Phase 5: Update tests
- [ ] Phase 6: Fix root login issue
- [ ] Phase 7: Documentation updates

## Background

### Scattered vs. Centralized Assessment
Authentication code is **moderately scattered** across 14 locations (better than pre-TEAM_143 udev):

**Well-Centralized:**
- PAM configs: `leviso/src/component/custom/pam.rs` (single file, 17+ configs)
- User management: `leviso/src/build/users.rs` (one module)
- Distro-spec: Single source of truth for component lists

**Moderately Scattered:**
- Live overlay: separate logic in `live.rs`
- Getty config: buried in `definitions.rs`
- SSH keys: in `etc.rs`
- Critical symlinks: `/usr/bin/login` in SBIN_BINARIES

**Needs Improvement:**
- Hidden dependencies (login symlink only documented in TEAM_108)
- Serial getty flags (`-L` flag critical but undocumented)
- No unified documentation

### All 14 Current Locations Handling Auth
1. `leviso/src/component/custom/pam.rs` - PAM creation
2. `leviso/profile/etc/pam.d/*` - 17 static PAM files
3. `leviso/src/build/users.rs` - passwd/shadow manipulation
4. `distro-spec/src/shared/users.rs` - UserSpec, UID constants
5. `leviso/profile/etc/{passwd,group,shadow,gshadow}` - Base users
6. `leviso/profile/live-overlay/etc/shadow` - Empty root password
7. `leviso/src/component/custom/live.rs` - Autologin services
8. `leviso/src/component/definitions.rs` - Getty + login symlink
9. `leviso/src/component/custom/etc.rs` - passwd, shadow, SSH keys
10. `distro-spec/src/shared/components.rs` - Component lists
11. `testing/fsdbg/src/checklist/auth_audit.rs` - Verification
12. `testing/rootfs-tests/tests/security.rs` - Runtime tests
13. `leviso/src/component/service.rs` - OPENSSH_SVC
14. `distro-spec/src/shared/auth/` - **NEW** (to be created)

### Known Issues
- **Root password on installed systems**: Base shadow has `root:!:...` (locked)
- **Live overlay mechanism**: Not documented why it only affects live ISO
- **Installation flow**: User must manually create user or set root password after install

## Implementation Plan

### Phase 1: Requirements Documentation
Create `distro-spec/src/shared/auth/requirements.md` with complete list of all login/auth requirements:
- 11 requirement categories (authentication, console, SSH, etc.)
- 60+ individual requirements
- Architecture documentation (OverlayFS three-layer)
- Why installed systems have locked root
- Verification checklist

### Phase 2: Auth Subsystem Structure
Create `distro-spec/src/shared/auth/` module:
- `mod.rs` - Public API
- `components.rs` - Component lists (from components.rs)
- `requirements.rs` - Requirements constants
- `pam.rs` - PAM config constants
- `getty.rs` - Getty config constants
- `ssh.rs` - SSH config constants

### Phase 3: Extract Constants
Move constants from leviso to distro-spec:
- PAM configs from `leviso/src/component/custom/pam.rs`
- Component lists from `distro-spec/src/shared/components.rs`
- Getty config from `leviso/src/component/definitions.rs`
- SSH config from `leviso/src/component/custom/etc.rs`

### Phase 4: Update Imports
Update consumers to import from `distro_spec::auth::*`:
- `leviso/src/component/custom/pam.rs`
- `leviso/src/component/definitions.rs`
- `leviso/src/component/custom/etc.rs`
- `testing/fsdbg/src/checklist/auth_audit.rs`

### Phase 5: Update Tests
Ensure all tests import from new location:
- `testing/fsdbg/` verification
- `testing/rootfs-tests/` runtime tests

### Phase 6: Fix Root Login Issue
Options:
- **Option A (Recommended)**: Prompt during recstrap for user creation (Arch-style)
- **Option B**: Auto-unlock root during installation
- **Option C**: Post-install script

Recommend Option A + document in installation guide.

### Phase 7: Documentation
Update docs to reference auth subsystem:
- Add CLAUDE.md section
- Update team files
- Add architecture diagrams to requirements.md

## Critical Files

### New Files (6)
- `distro-spec/src/shared/auth/mod.rs`
- `distro-spec/src/shared/auth/components.rs`
- `distro-spec/src/shared/auth/requirements.md`
- `distro-spec/src/shared/auth/pam.rs`
- `distro-spec/src/shared/auth/getty.rs`
- `distro-spec/src/shared/auth/ssh.rs`

### Modified Files (7)
- `distro-spec/src/shared/mod.rs`
- `distro-spec/src/shared/components.rs`
- `leviso/src/component/custom/pam.rs`
- `leviso/src/component/definitions.rs`
- `leviso/src/component/custom/etc.rs`
- `testing/fsdbg/src/checklist/auth_audit.rs`
- `leviso/profile/etc/motd` (optional)

## Key Insights

### OverlayFS Three-Layer Mount
```
Layer 3 (top):    tmpfs (/overlay/upper)         [read-write, ephemeral]
Layer 2 (middle): /live/overlay from ISO         [read-only, live configs]
Layer 1 (bottom): EROFS (/rootfs)                [read-only, base system]
```

Result: Files in `/live/overlay` override base files.
- `/live/overlay/etc/shadow` (empty root) overrides `/rootfs/etc/shadow` (locked)
- **Installed systems don't use overlay**: recstrap extracts EROFS only

### Why Root Is Locked on Installed Systems
1. Live ISO: OverlayFS merges EROFS + live-overlay + tmpfs
2. Live overlay has empty root password in `/live/overlay/etc/shadow`
3. Installation: recstrap extracts EROFS only (not live-overlay)
4. Result: Installed system has locked root from EROFS base
5. **Solution**: User must create initial user OR set root password after install

## Success Criteria
- ✅ All auth constants in `distro-spec/src/shared/auth/`
- ✅ No duplication of component lists
- ✅ All tests pass
- ✅ ISO builds successfully
- ✅ Requirements documented
- ✅ Critical dependencies documented in code
- ✅ Root login issue resolved (user creation prompt or documentation)

## Implementation Progress

### Phase 1, 2 & 3 Complete ✅

**MAJOR MILESTONE**: Auth subsystem fully extracted and consolidated

### Detailed Progress

#### Phase 1 & 2 Complete ✅

**Created files**:
```
distro-spec/src/shared/auth/
├── mod.rs              # Public API + re-exports (300+ lines)
├── requirements.md     # Complete requirements doc (700+ lines)
├── components.rs       # Component lists (200+ lines)
├── pam.rs             # 12 PAM config files (400+ lines)
├── getty.rs           # Getty/console config (80+ lines)
└── ssh.rs             # SSH server config (120+ lines)
```

**Key decisions made**:
1. **PAM configs moved**: All 12+ PAM file contents now in `distro-spec/src/shared/auth/pam.rs` with full documentation
2. **Component lists**: AUTH_BIN, AUTH_SBIN, SHADOW_SBIN, SSH_BIN/SBIN, PAM_MODULES, PAM_CONFIGS, SECURITY_FILES all defined in `components.rs`
3. **Re-exports**: All public APIs in `mod.rs` with clear documentation about what goes where
4. **Backwards compatibility**: Will add re-exports to `distro-spec/src/shared/components.rs` to avoid breaking changes
5. **Build logic stays in leviso**: pam.rs creation functions remain in `leviso/src/component/custom/pam.rs` (not moved)

### Phase 3 Complete ✅

**Consolidated constants**:
1. **From components.rs** → **auth/components.rs**:
   - AUTH_BIN (4 items: su, sudo, sudoedit, sudoreplay)
   - AUTH_SBIN (2 items: visudo, unix_chkpwd)
   - SHADOW_SBIN (12 items: faillock, chage, newusers, etc.)
   - SSH_BIN (6 items: ssh, scp, sftp, ssh-keygen, ssh-add, ssh-agent)
   - SSH_SBIN (1 item: sshd)
   - SUDO_LIBS (6 items: libsudo_util variants, sudoers.so, etc.)
   - PAM_MODULES (40+ modules: pam_unix.so, pam_permit.so, etc.)
   - PAM_CONFIGS (23 files: login, sshd, sudo, su, passwd, etc.)
   - SECURITY_FILES (8 files: limits.conf, faillock.conf, etc.)

2. **Backwards Compatibility**: All constants re-exported from components.rs using `pub use super::auth::components::*;`
   - Existing code continues to work: `distro_spec::shared::AUTH_BIN`
   - New code can use: `distro_spec::shared::auth::components::AUTH_BIN`

3. **Build Status**:
   - ✅ `cargo check` passes with no errors
   - ✅ All existing imports still work (backwards compatible)
   - ✅ distro-spec compiles successfully
   - ✅ No circular dependencies

**What's NOT yet done**:
- leviso doesn't need to change imports (uses distro_spec::shared which still exports these)
- pam.rs creation functions in leviso remain local (not moved - build logic stays in leviso)
- testing/fsdbg could optionally import from auth module directly (optional optimization)

### Next Steps

**Phase 3: Extract constants**
- Move AUTH_BIN, AUTH_SBIN, etc. from `components.rs` to `auth/components.rs`
- Update `components.rs` to re-export from auth module
- Update imports in `leviso/src/component/definitions.rs`

**Phase 4: Update leviso imports**
- `leviso/src/component/custom/pam.rs`: Optionally use pam constants from auth module
- `leviso/src/component/definitions.rs`: Use getty constants
- `leviso/src/component/custom/etc.rs`: Use ssh constants

**Phase 5: Update tests**
- `testing/fsdbg/` - Import from auth module instead of components
- `testing/rootfs-tests/` - Update verification to use auth constants

**Phase 6: Root login issue**
- Implement user creation prompt in recstrap (Option A from requirements)
- OR document post-install steps (Option C from requirements)

**Phase 7: Update documentation**
- CLAUDE.md - Add auth subsystem section
- TEAM file - Document decisions and lessons learned

## Notes
- Keep pam.rs creation functions in leviso (build-time logic)
- Only move constants and documentation
- Add backwards compatibility re-exports in components.rs
- This is data consolidation, not architecture change
- Compilation checks: ✅ distro-spec builds successfully
- distro-spec pulls in auth module automatically via shared/mod.rs
- leviso not yet updated but will compile once imports are added

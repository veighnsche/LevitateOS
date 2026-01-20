# TEAM_059: Apply Rich Text Features to Docs Content Files 04-06

## Status: Complete

## Goal
Convert docs content files 04, 05, and 06 to use the new block types from file 03:
- `type: "command"` with `description`, `command`, and optional `output`
- Add `bold` import where needed

## Files to Modify
1. `04-installation-base.ts` - Extract Stage3, Generate fstab, Enter chroot
2. `05-installation-config.ts` - Timezone, Locale, Hostname, Password, User
3. `06-installation-boot.ts` - Bootloader, Services, Reboot, Troubleshooting

## Pattern
**Before:**
```typescript
{
  type: "code",
  language: "bash",
  content: `# Comment describing command
actual-command`,
}
```

**After:**
```typescript
{
  type: "command",
  description: "Comment describing command",
  command: "actual-command",
},
```

## Key Rules
- Keep `type: "code"` for file content blocks (fstab, loader.conf, etc.)
- Multi-command sequences become multiple `command` blocks or array syntax
- `interactive` type is for truly interactive tools like fdisk

## Progress
- [x] Convert 04-installation-base.ts
- [x] Convert 05-installation-config.ts
- [x] Convert 06-installation-boot.ts
- [x] Run typecheck to verify

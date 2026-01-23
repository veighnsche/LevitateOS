# TEAM_049: Rich Text Tagged Templates

## Goal
Implement a tagged template literal system for declaring rich text (links, bold, code) in TypeScript docs-content with full type safety.

## Design

### Types
```typescript
type InlineNode = string | InlineLink | InlineBold | InlineCode | InlineItalic

interface InlineLink { type: 'link'; text: string; href: string }
interface InlineBold { type: 'bold'; text: string }
interface InlineCode { type: 'code'; text: string }
interface InlineItalic { type: 'italic'; text: string }

type RichText = InlineNode[]
```

### Helper Functions
```typescript
const link = (text: string, href: string): InlineLink => ({ type: 'link', text, href })
const bold = (text: string): InlineBold => ({ type: 'bold', text })
const code = (text: string): InlineCode => ({ type: 'code', text })
const italic = (text: string): InlineItalic => ({ type: 'italic', text })
```

### Tagged Template
```typescript
function rich(strings: TemplateStringsArray, ...values: InlineNode[]): RichText {
  const result: RichText = []
  strings.forEach((str, i) => {
    if (str) result.push(str)
    if (i < values.length) result.push(values[i])
  })
  return result
}
```

### Usage
```typescript
// Before (inline markdown)
{ type: "text", content: "Use [Rufus](https://rufus.ie) with **DD mode**." }

// After (typed)
{ type: "text", content: rich`Use ${link("Rufus", "https://rufus.ie")} with ${bold("DD mode")}.` }
```

## Files to Create/Modify
- `docs-content/src/rich-text.ts` - Types and helpers
- `docs-content/src/types.ts` - Update TextBlock to accept RichText
- `website/src/components/docs/RichText.tsx` - Renderer component

## Status
- [x] Design approved
- [x] Implement rich-text.ts
- [x] Update types
- [x] Create renderer
- [x] Migrate one content file as example

## Summary

Implemented tagged template literal system for type-safe rich text in docs:

**Files Created/Modified:**
- `docs-content/src/rich-text.ts` - Types and helpers (`rich`, `link`, `bold`, `code`, `italic`)
- `docs-content/src/types.ts` - Updated TextBlock to accept `string | RichText`
- `docs-content/src/index.ts` - Exported new types and helpers
- `website/src/components/docs/DocsPage.tsx` - Added `InlineNodeRenderer` for RichText arrays
- `website/src/components/docs/index.ts` - Re-exported types and helpers
- `docs-content/src/content/01-getting-started/01-getting-started.ts` - Example migration

**Usage:**
```typescript
import { rich, link, bold, code } from "@levitate/docs-content"

{ type: "text", content: rich`Use ${link("Rufus", "https://rufus.ie")} with ${bold("DD mode")}.` }
```

Plain markdown strings also accepted.

## Full Migration Complete

All docs migrated to rich text system:

**Getting Started:**
- `01-getting-started.ts` ✓
- `02-installation.ts` ✓

**Package Manager:**
- `01-cli-reference.ts` ✓
- `02-recipe-format.ts` ✓

**Helpers:**
- `01-overview.ts` ✓
- `02-acquire.ts` ✓
- `03-build.ts` ✓
- `04-install.ts` ✓
- `05-filesystem.ts` ✓
- `06-environment.ts` ✓
- `07-commands.ts` ✓
- `08-http.ts` ✓

**Types updated:**
- `DocsContent.intro` now accepts `string | RichText`
- `ListBlock.items` now accepts `string | RichText | ListItem`
- `ListItem.text` and `ListItem.children` now accept `string | RichText`

**Renderers updated:**
- `IntroRenderer` handles RichText arrays
- `TextBlockRenderer` handles RichText arrays
- `ListBlockRenderer` handles RichText arrays via `InlineContentRenderer`

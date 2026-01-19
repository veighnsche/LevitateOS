# TEAM_052: Syntax Highlighting for Code Blocks

## Goal
Add colored syntax highlighting for bash, rhai, json, yaml, toml in CodeBlock and FileBlock components.

## Approach
Using Shiki (industry standard, VS Code grammar engine) with CSS variable theming.

## Files to Modify
1. `website/package.json` - Add shiki dependency
2. `website/src/lib/highlighter.ts` (NEW) - Shiki singleton + highlight function
3. `website/src/components/CodeBlock.tsx` - Add highlighting with useEffect
4. `website/src/components/docs/DocsPage.tsx` - Add highlighting to FileBlockRenderer
5. `website/src/styles.css` - Add Shiki CSS variables for OKLCH theme

## Status
- [ ] Install shiki
- [ ] Create highlighter singleton
- [ ] Update CodeBlock
- [ ] Update FileBlockRenderer in DocsPage
- [ ] Add CSS variables
- [ ] Test

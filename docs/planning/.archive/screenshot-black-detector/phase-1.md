# Phase 1: Discovery - Screenshot Black Screen Detector

**Feature:** Auto-detect black/empty screenshots in xtask  
**TEAM:** 329

---

## Problem Statement

When running screenshot tests (`cargo xtask test levitate`), the tool captures screenshots but doesn't analyze their content. Teams must manually open images to determine if displays are working or showing black screens.

**Pain point:** TEAM_328 had to visually inspect screenshots to confirm x86_64 was still showing a black screen.

## Who Benefits

- **Developers** investigating display issues
- **CI/CD pipelines** that need automated pass/fail detection
- **Future teams** running display verification tests

## Success Criteria

1. ✅ Test output clearly indicates when a screenshot is black/empty
2. ✅ Test output indicates when a screenshot has content (non-black)
3. ✅ No false positives on legitimate dark screenshots with some content
4. ✅ Detection works for both PPM and PNG formats

## Current State

### How it works today

```rust
// xtask/src/tests/screenshot.rs
fn take_screenshot(client: &mut QmpClient, output: &str) -> Result<()> {
    // Takes screenshot via QMP
    // Converts PPM to PNG
    // Returns Ok() without analyzing content
}
```

The test reports success if:
- Screenshot file was created
- File is not empty (size > 0)

**No content analysis is performed.**

### Evidence of the problem

```
━━━ Results ━━━
  ✅ aarch64: Screenshot captured   # 8.3 KB - has content
  ✅ x86_64: Screenshot captured    # 432 bytes - SOLID BLACK
```

Both show ✅ even though x86_64 is completely black.

## Codebase Areas

| File | Relevance |
|------|-----------|
| `xtask/src/tests/screenshot.rs` | Main screenshot capture logic |
| `xtask/src/tests/common.rs` | Shared test utilities |
| `tests/screenshots/*.png` | Output location |

## Constraints

1. **Performance**: Analysis should be fast (< 100ms per image)
2. **Dependencies**: Prefer using existing `image` crate or simple analysis
3. **Threshold**: Need sensible definition of "black" (not just 0,0,0 pixels)

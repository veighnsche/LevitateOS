# Phase 2: Design - Screenshot Black Screen Detector

**Feature:** Auto-detect black/empty screenshots in xtask  
**TEAM:** 329

---

## Proposed Solution

Add an `analyze_screenshot()` function that:
1. Reads the PNG/PPM image
2. Calculates average brightness or samples pixels
3. Returns a verdict: `HasContent` or `Black`
4. Updates test output to show appropriate icon

### User-Facing Behavior

**Before:**
```
  ✅ x86_64: Screenshot captured
```

**After:**
```
  ⚠️  x86_64: Screenshot captured (BLACK SCREEN DETECTED)
```
or
```
  ✅ x86_64: Screenshot captured (display working)
```

---

## API Design

```rust
/// Result of screenshot content analysis
pub enum ScreenshotContent {
    /// Image has visible content (text, graphics)
    HasContent { brightness: f32 },
    /// Image is black or nearly black
    Black { brightness: f32 },
}

/// Analyze a screenshot to detect if it's black/empty
/// 
/// Returns Black if average brightness < threshold (default: 5/255)
pub fn analyze_screenshot(path: &str) -> Result<ScreenshotContent>;
```

---

## Detection Algorithm

### Option A: Average Brightness (Recommended)
```
1. Load image
2. Convert to grayscale (or use luminance: 0.299*R + 0.587*G + 0.114*B)
3. Calculate average pixel value (0-255)
4. If average < THRESHOLD (e.g., 5) → Black
5. Else → HasContent
```

**Pros:** Simple, fast, handles slight variations  
**Cons:** Could miss very dark legitimate content

### Option B: Sample-Based Detection
```
1. Sample pixels at fixed positions (corners, center, random)
2. If all samples < threshold → Black
```

**Pros:** Very fast  
**Cons:** Could miss small content areas

### Chosen Approach: Option A

Average brightness is robust and the `image` crate makes it trivial.

---

## Threshold Decision

| Threshold | Meaning |
|-----------|---------|
| 0 | Only pure black (0,0,0) |
| 5 | Very dark (allows for compression artifacts) |
| 10 | Dark with some noise tolerance |

**Recommendation:** Use threshold of **5** (out of 255).

This catches:
- Pure black screens
- Near-black screens (compression artifacts)
- QEMU uninitialized framebuffers

---

## Implementation Location

```rust
// xtask/src/tests/screenshot.rs

fn analyze_screenshot(path: &str) -> Result<ScreenshotContent> {
    let img = image::open(path)?;
    let gray = img.to_luma8();
    
    let sum: u64 = gray.pixels().map(|p| p.0[0] as u64).sum();
    let count = gray.width() as u64 * gray.height() as u64;
    let avg = (sum / count) as f32;
    
    if avg < 5.0 {
        Ok(ScreenshotContent::Black { brightness: avg })
    } else {
        Ok(ScreenshotContent::HasContent { brightness: avg })
    }
}
```

---

## Integration Points

### 1. After `take_screenshot()` call

```rust
take_screenshot(&mut client, &screenshot)?;
let content = analyze_screenshot(&png)?;
match content {
    ScreenshotContent::Black { .. } => {
        println!("[{}] ⚠️  BLACK SCREEN DETECTED", arch);
    }
    ScreenshotContent::HasContent { .. } => {
        println!("[{}] ✅ Display has content", arch);
    }
}
```

### 2. In results summary

Update the results section to show detection status.

---

## Dependencies

The `image` crate is likely already in `xtask/Cargo.toml` (used by ImageMagick fallback). If not:

```toml
[dependencies]
image = "0.24"
```

---

## Open Questions

None - this is a straightforward feature with clear requirements.

---

## Phases Summary

This is a **small feature** - single phase implementation:

1. ✅ Phase 1: Discovery (this document)
2. ✅ Phase 2: Design (this document)  
3. **Phase 3: Implementation** - Single UoW, ~50 lines of code

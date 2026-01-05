# levitate-terminal

Platform-agnostic ANSI terminal emulator for LevitateOS — handles text rendering, cursor management, and ANSI escape sequences.

## Purpose

This crate provides a **standalone terminal emulator** that is independent of any specific hardware. It can render to any object that implements the `embedded-graphics` `DrawTarget` trait, making it suitable for both the GPU console and potential future use cases (like a windowing system or serial multiplexer).

## Architecture

```
levitate-terminal/src/
└── lib.rs          # Terminal state, rendering logic, and ANSI parsing
```

## Key Components

### Terminal

The core structure managing the emulator state:
- **Dimensions**: Calculated based on font metrics and screen size.
- **Cursor Management**: Tracks position, visibility, and supports "exclusive" cursor rendering (hiding/showing during writes).
- **ANSI Engine**: A state machine for processing standard ANSI escape sequences (e.g., clearing screen).
- **Font Support**: Uses `profont` for high-quality, readable text on bare metal.

## Features

- **ANSI Support**: Basic CSI sequences (`[J` for clear screen).
- **Automatic Wrapping**: Handles line breaks and carriage returns.
- **Platform Agnostic**: Works with any `DrawTarget<Color = Rgb888>`.
- **Fast Rendering**: Optimized for direct framebuffer access via `embedded-graphics`.

## Usage

```rust
use levitate_terminal::Terminal;
use embedded_graphics::prelude::*;

// Create terminal based on screen size
let mut terminal = Terminal::new(1280, 800);

// Write text to any DrawTarget (like levitate-gpu::Display)
terminal.write_str(&mut display, "Hello, LevitateOS!\n");
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `embedded-graphics` | Drawing traits and text rendering |
| `profont` | Embedded-friendly monospace font |

## Integration with LevitateOS

The kernel integrates this crate in `kernel/src/terminal.rs`, where it is paired with `levitate-gpu` to provide the primary boot console. It is also used to mirror UART output to the screen.

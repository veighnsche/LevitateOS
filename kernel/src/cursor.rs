use crate::gpu::Display;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use levitate_utils::Spinlock;

// TEAM_058: Track both current and previous position to erase trails
struct CursorState {
    x: i32,
    y: i32,
    prev_x: i32,
    prev_y: i32,
    saved_pixels: [[Rgb888; 10]; 10],
    has_saved: bool,
}

static CURSOR: Spinlock<CursorState> = Spinlock::new(CursorState { 
    x: 500, 
    y: 500,
    prev_x: 500,
    prev_y: 500,
    saved_pixels: [[Rgb888::BLACK; 10]; 10],
    has_saved: false,
});

#[allow(dead_code)]
pub fn update(x: i32, y: i32) {
    let mut state = CURSOR.lock();
    state.prev_x = state.x;
    state.prev_y = state.y;
    state.x = x;
    state.y = y;
}

pub fn set_x(x: i32) {
    let mut state = CURSOR.lock();
    state.prev_x = state.x;
    state.x = x;
}

pub fn set_y(y: i32) {
    let mut state = CURSOR.lock();
    state.prev_y = state.y;
    state.y = y;
}

pub fn draw(display: &mut Display) {
    let mut state = CURSOR.lock();
    
    // TEAM_059: Restore previous pixels instead of drawing BLACK
    if state.has_saved {
        let mut guard = crate::gpu::GPU.lock();
        if let Some(gpu) = guard.as_mut() {
            let (width, height) = gpu.dimensions();
            let fb = gpu.framebuffer();
            
            for dy in 0..10 {
                for dx in 0..10 {
                    let py = state.prev_y + dy;
                    let px = state.prev_x + dx;
                    
                    if py >= 0 && py < height as i32 && px >= 0 && px < width as i32 {
                        let idx = (py as usize * width as usize + px as usize) * 4;
                        let color = state.saved_pixels[dy as usize][dx as usize];
                        fb[idx] = color.r();
                        fb[idx + 1] = color.g();
                        fb[idx + 2] = color.b();
                        fb[idx + 3] = 255;
                    }
                }
            }
            gpu.flush();
        }
    }
    
    // TEAM_059: Save new pixels before drawing white cursor
    {
        let mut guard = crate::gpu::GPU.lock();
        if let Some(gpu) = guard.as_mut() {
            let (width, height) = gpu.dimensions();
            let fb = gpu.framebuffer();
            
            for dy in 0..10 {
                for dx in 0..10 {
                    let py = state.y + dy;
                    let px = state.x + dx;
                    
                    if py >= 0 && py < height as i32 && px >= 0 && px < width as i32 {
                        let idx = (py as usize * width as usize + px as usize) * 4;
                        state.saved_pixels[dy as usize][dx as usize] = Rgb888::new(fb[idx], fb[idx+1], fb[idx+2]);
                    } else {
                        state.saved_pixels[dy as usize][dx as usize] = Rgb888::BLACK;
                    }
                }
            }
            state.has_saved = true;
        }
    }
    
    // Draw new cursor position (white)
    let _ = Rectangle::new(Point::new(state.x, state.y), Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
        .draw(display);

    // TEAM_059: Explicitly flush after drawing cursor
    crate::gpu::GPU.lock().as_mut().map(|s| s.flush());
}

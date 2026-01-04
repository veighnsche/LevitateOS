use crate::gpu::Display;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use levitate_utils::Spinlock;

struct CursorState {
    x: i32,
    y: i32,
}

static CURSOR: Spinlock<CursorState> = Spinlock::new(CursorState { x: 500, y: 500 });

#[allow(dead_code)]
pub fn update(x: i32, y: i32) {
    let mut state = CURSOR.lock();
    state.x = x;
    state.y = y;
}

pub fn set_x(x: i32) {
    let mut state = CURSOR.lock();
    state.x = x;
}

pub fn set_y(y: i32) {
    let mut state = CURSOR.lock();
    state.y = y;
}

pub fn draw(display: &mut Display) {
    let state = CURSOR.lock();
    // Simple 10x10 white cursor
    let _ = Rectangle::new(Point::new(state.x, state.y), Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
        .draw(display);
}

//! xHCI USB Controller Driver for LevitateOS
//!
//! This is a stub implementation for the xHCI USB controller.

#![no_std]

use input_device::{InputDevice, InputEvent};

/// xHCI Controller
pub struct XhciController {
    // TODO: Add controller state and rings
}

impl XhciController {
    /// Create a new xHCI controller stub
    pub fn new() -> Self {
        Self {}
    }
}

impl InputDevice for XhciController {
    fn poll(&mut self) -> bool {
        false
    }

    fn read_char(&mut self) -> Option<char> {
        None
    }

    fn ctrl_c_pressed(&self) -> bool {
        false
    }

    fn poll_event(&mut self) -> Option<InputEvent> {
        None
    }
}

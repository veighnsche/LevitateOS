//! Centralized panic handling for LevitateOS userspace programs.

use core::panic::PanicInfo;
use libsyscall::common_panic_handler;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

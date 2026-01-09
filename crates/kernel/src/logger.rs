//! TEAM_221: Kernel Logger implementation.
//!
//! Implements the `log::Log` trait to route log messages to the serial console.
//! Supports compile-time and runtime log level filtering.

use log::{Level, LevelFilter, Metadata, Record};
use los_hal::println;

/// Global logger instance
static LOGGER: SimpleLogger = SimpleLogger;

/// Simple Logger implementation
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // TEAM_272: Filter out noisy logs from external crates to match golden boot logs
            let target = record.metadata().target();
            if target.starts_with("virtio_drivers") {
                return;
            }

            // TEAM_272: Remove level prefix to match golden boot logs
            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}

/// Initialize the logger.
///
/// # Arguments
/// * `max_level` - The maximum log level to display.
pub fn init(max_level: LevelFilter) {
    log::set_logger(&LOGGER).expect("Failed to set logger");
    log::set_max_level(max_level);
    // TEAM_272: Removed initialization message to match golden boot logs
}

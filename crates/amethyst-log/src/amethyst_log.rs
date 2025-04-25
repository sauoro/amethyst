extern crate core;

use chrono::Local;
use log::{Level, Log};

// TODO: make a better logging system.
pub const AMETHYST_LOGGER: AmethystLogger = AmethystLogger;

pub struct AmethystLogger;

impl Log for AmethystLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let now = Local::now();
            println!(
                "{} {} {}",
                now.format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

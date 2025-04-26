use chrono::Utc;
use log::{Level, Log, LevelFilter};
use std::io::{BufWriter, Write};
use std::sync::Mutex;

pub struct AmethystLogger {
    max_level: Level,
    writer: Mutex<BufWriter<std::io::Stdout>>,
}

impl AmethystLogger {
    pub fn new(max_level: Level) -> Self {
        AmethystLogger {
            max_level,
            writer: Mutex::new(BufWriter::new(std::io::stdout())),
        }
    }

    pub fn init(max_level: Level) -> Result<(), log::SetLoggerError> {
        let logger = AmethystLogger::new(max_level);
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(max_level.to_level_filter());
        Ok(())
    }
}

impl Log for AmethystLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let now = Utc::now();
            let message = format!(
                "{} {} [{}] {}\n",
                now.format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.args()
            );
            let mut writer = self.writer.lock().unwrap();
            writer.write_all(message.as_bytes()).unwrap();
        }
    }

    fn flush(&self) {
        let mut writer = self.writer.lock().unwrap();
        writer.flush().unwrap();
    }
}

fn main() {
    AmethystLogger::init(Level::Info).unwrap();
    log::info!("Server started");
    log::debug!("This won't be logged because level is Info");
    log::error!("An error occurred");
    log::logger().flush();
}

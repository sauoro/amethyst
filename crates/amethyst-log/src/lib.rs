use chrono::{Local};
use log::{Level, Log, set_max_level, set_boxed_logger};
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
        set_boxed_logger(Box::new(logger))?;
        set_max_level(max_level.to_level_filter());
        Ok(())
    }
}


impl Log for AmethystLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let now = Local::now();
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

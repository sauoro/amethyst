use chrono::Local;
use log::{set_boxed_logger, set_max_level, Level, Log, SetLoggerError};
use std::io::{stdout, BufWriter, Write};
use std::sync::mpsc;
use std::thread;

pub enum LogCommand {
    Record(String),
    Flush,
    Terminate,
}

pub struct AmethystLogger {
    max_level: Level,
    sender: mpsc::SyncSender<LogCommand>,
}

impl AmethystLogger {
    pub fn new(max_level: Level, buffer_size: usize) -> (Self, mpsc::Receiver<LogCommand>) {
        let (sender, receiver) = mpsc::sync_channel(buffer_size);

        let logger = AmethystLogger { max_level, sender };
        (logger, receiver)
    }

    pub fn init(max_level: Level, buffer_size: usize) -> Result<(), SetLoggerError> {
        let (logger, receiver) = AmethystLogger::new(max_level, buffer_size);

        let _handle = thread::Builder::new()
            .name("amethyst-log-writer".into())
            .spawn(move || {
                let mut writer = BufWriter::new(stdout());
                while let Ok(command) = receiver.recv() {
                    match command {
                        LogCommand::Record(message) => {
                            if let Err(e) = writer.write_all(message.as_bytes()) {
                                eprintln!("[AmethystLogger] Failed to write log record: {}", e);
                            }
                        }
                        LogCommand::Flush => {
                            if let Err(e) = writer.flush() {
                                eprintln!("[AmethystLogger] Failed to flush log: {}", e);
                            }
                        }
                        LogCommand::Terminate => {
                            let _ = writer.flush();
                            break; // Exit the loop
                        }
                    }
                }
                // Channel closed or termination requested. Ensure final flush.
                let _ = writer.flush();
            })
            .expect("Failed to spawn logger thread");

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

            if let Err(e) = self.sender.try_send(LogCommand::Record(message)) {
                eprintln!("[AmethystLogger] Failed to send log message: {}", e);
            }
        }
    }

    fn flush(&self) {
        let _ = self.sender.send(LogCommand::Flush);
    }
}

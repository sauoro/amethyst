use chrono::Local;

const ANSI_RESET: &str = "\x1b[0m";
const AMETHYST_COLOR: &str = "\x1b[38;5;171m";
const YELLOW_COLOR: &str = "\x1b[38;5;226m";
const RED_COLOR: &str = "\x1b[38;5;196m";

pub struct Logger {
    name: Option<String>,
}

impl Logger {
    pub fn new(name: String) -> Self {
        Self { name: Some(name) }
    }

    pub fn nameless() -> Self {
        Self { name: None }
    }

    pub(crate) fn raw_log(&self, msg: &str, l_type: &str, color_format: Option<&str>) {
        let now = Local::now();
        let timestamp = now.format("%H:%M:%S%.3f");
        let format = format!("[{} {}]", timestamp, l_type);
        let color = color_format.unwrap_or_else(|| ANSI_RESET);
        match &self.name {
            Some(name) => println!(
                "{}{} {} {}{}",
                &color,
                format,
                format!("[{}]", name),
                msg,
                ANSI_RESET
            ),
            None => println!(
                "{}{} {}{}",
                &color,
                format,
                msg,
                ANSI_RESET
            ),
        }
    }
}

pub trait Log {
    fn log(&self, msg: &str, l_type: &str, color_format: Option<&str>);

    fn info(&self, msg: &str) {
        self.log(&msg, "INFO", Some(ANSI_RESET));
    }

    fn warn(&self, msg: &str) {
        self.log(&msg, "WARN", Some(YELLOW_COLOR));
    }

    fn error(&self, msg: &str) {
        self.log(&msg, "ERROR", Some(RED_COLOR));
    }
}

impl Log for Logger {
    fn log(&self, msg: &str, l_type: &str, color_format: Option<&str>) {
        self.raw_log(msg, l_type, color_format);
    }
}

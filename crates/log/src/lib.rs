use chrono::Local;

pub struct Logger {
    name: Option<String>,
}

impl Logger {
    pub fn new(name: String) -> Self {
        Self {
            name: Some(name),
        }
    }
    
    pub fn nameless() -> Self {
        Self { name: None }
    }
    

    pub(crate) fn raw_log(&self, msg: &str) {
        let now = Local::now();
        let timestamp = now.format("[%H:%M:%S%.3f]");
        let formatted_timestamp = format!("{}{}{}", "\x1b[38;5;45m", timestamp, "\x1b[0m");
        match &self.name {
            Some(name) => println!("{} {} {}", formatted_timestamp, format!("{}{}{}{}{}", "\x1b[38;5;171m", "[", name, "]", "\x1b[0m"), msg),
            None => println!("{} {}", formatted_timestamp, msg),
        }
    }
}

pub trait Log {
    fn log(&self, msg: &str);
    
    fn info(&self, msg: &str) {
        let formatted_message = format!("{}[INFO] {}{}", "\x1b[1;32m", msg, "\x1b[0m");
        self.log(&formatted_message);
    }
    
    fn warn(&self, msg: &str) {
        let formatted_message = format!("{}[WARN] {}{}", "\x1b[1;33m", msg, "\x1b[0m");
        self.log(&formatted_message);
    }
    
    fn error(&self, msg: &str) {
        let formatted_message = format!("{}[ERROR] {}{}", "\x1b[1;31m", msg, "\x1b[0m");
        self.log(&formatted_message);
    }
}

impl Log for Logger {
    fn log(&self, msg: &str) {
        self.raw_log(msg);
    }
}
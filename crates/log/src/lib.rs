use chrono::Local;

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

    pub(crate) fn raw_log(&self, msg: &str, l_type: &str) {
        let now = Local::now();
        let timestamp = now.format("%H:%M:%S");
        let format = format!("({} {})", timestamp, l_type);
        match &self.name {
            Some(name) => println!("{} {} {}", format, format!("[{}]", name), msg,),
            None => println!("{} {}", format, msg,),
        }
    }
}

pub trait Log {
    fn log(&self, msg: &str, l_type: &str);

    fn info(&self, msg: &str) {
        self.log(&msg, "INFO");
    }

    fn debug(&self, msg: &str) {
        self.log(&msg, "DEBUG");
    }

    fn warn(&self, msg: &str) {
        self.log(&msg, "WARN");
    }

    fn error(&self, msg: &str) {
        self.log(&msg, "ERROR");
    }
}

impl Log for Logger {
    fn log(&self, msg: &str, l_type: &str) {
        self.raw_log(msg, l_type);
    }
}

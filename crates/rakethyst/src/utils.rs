// src/utils.rs
use std::sync::OnceLock;
use std::time::Instant;

static SERVER_START_TIME: OnceLock<Instant> = OnceLock::new();

pub fn init_time() {
    SERVER_START_TIME.get_or_init(Instant::now);
}

pub fn cur_time_millis() -> u64 {
    SERVER_START_TIME
        .get()
        .expect("Time not initialized. Call init_time() first.")
        .elapsed()
        .as_millis() as u64
}

pub fn cur_time_secs() -> u64 {
    SERVER_START_TIME
        .get()
        .expect("Time not initialized. Call init_time() first.")
        .elapsed()
        .as_secs()
}
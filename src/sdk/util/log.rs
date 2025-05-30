use log::{info, warn, error, debug};
use std::env;

pub fn init_logging() {
    let default = "info";
    let level = env::var("RUST_LOG").unwrap_or_else(|_| default.to_string());
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();
}

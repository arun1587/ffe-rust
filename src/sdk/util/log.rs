use std::env;
use env_logger::{Builder, Env};

pub fn init_logging() {
    let default = "info";
    let level = env::var("RUST_LOG").unwrap_or_else(|_| default.to_string());
    Builder::from_env(Env::default().default_filter_or(level))
        .format_timestamp_secs()
        .format_module_path(false)
        .init();
}

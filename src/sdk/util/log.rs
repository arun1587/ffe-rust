use env_logger::{Builder, Env};
use log::LevelFilter;
use std::io::Write;

pub fn init_logging() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            // Use a specific color for each log level
            let level_style = buf.default_level_style(record.level());

            // Format the timestamp
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

            // Write the formatted log record
            writeln!(
                buf,
                "[{timestamp} {level_style}{level:<5}{level_style:#}] {args}",
                level = record.level(),
                args = record.args()
            )
        })
        .filter(None, LevelFilter::Info) // Set a default level
        .init();
}

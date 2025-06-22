use chrono::Datelike;
use clap::Parser;
use ffe_rust::{
    sdk::config::OrsConfig,
    sdk::departments::DepartmentLookup,
    sdk::events::{filter_reachable_events, get_events_for_month},
    sdk::routing::{cache::GeoCache, provider::RemoteOrsProvider},
    sdk::util::{log::init_logging, rate_limit::Limiter},
};
use reqwest::blocking::Client as HttpClient;
use std::{error::Error, fs::File, io::Write};

/// A CLI tool to find reachable FFE chess tournaments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The origin city name (e.g., "Rennes")
    #[arg(short, long)]
    city: String,

    /// The 2-digit department code of the origin city (e.g., 35)
    #[arg(short, long)]
    department: String,

    /// The month to search for events (1-12)
    #[arg(short, long)]
    month: u32,

    /// [Optional] Maximum travel time in hours
    #[arg(long, default_value_t = 1.5)]
    max_hours: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Start with our custom logger
    init_logging();
    dotenvy::dotenv().ok();

    // --- 1. Argument Parsing with Clap ---
    // This is now robust, typed, and provides help messages!
    let cli = Cli::parse();

    // Intelligently determine the year based on the current date
    let current_date = chrono::Local::now().date_naive();
    let year = if cli.month < current_date.month() {
        current_date.year() + 1
    } else {
        current_date.year()
    };
    log::info!(
        "Searching for events in month {} of year {}",
        cli.month,
        year
    );

    // --- 2. Dependency Initialization ---
    let config = OrsConfig::from_env()?;
    let limiter = Limiter::new();
    let provider = match config {
        OrsConfig::Remote { api_key } => RemoteOrsProvider::new(api_key, limiter),
        OrsConfig::Local { .. } => todo!(),
    };

    let department_lookup = DepartmentLookup::new("src/departments.csv")?;
    let mut cache = GeoCache::load_from_file("geo_cache.json")?;
    let http_client = HttpClient::new();

    let origin_query = department_lookup
        .build_geocode_query(&cli.city, &cli.department)
        .ok_or_else(|| format!("Unknown department code: {}", cli.department))?;
    log::info!("Origin location set to: {}", origin_query);

    // --- 3. Execute SDK Logic ---
    let all_events = get_events_for_month(cli.month, year, &http_client, &department_lookup)?;
    log::info!(
        "Found {} total events in France for {}/{}",
        all_events.len(),
        cli.month,
        year
    );

    let reachable_events = filter_reachable_events(
        &cli.city,
        &origin_query,
        &all_events,
        &department_lookup,
        &provider,
        &mut cache,
        cli.max_hours,
    );

    // --- 4. Output Results ---
    log::info!(
        "Found {} events reachable from {} within {} hours.",
        reachable_events.len(),
        cli.city,
        cli.max_hours
    );

    let json_output = serde_json::to_string_pretty(&reachable_events)?;
    let mut file = File::create("reachable_events.json")?;
    file.write_all(json_output.as_bytes())?;
    log::info!("✅ Reachable events written to reachable_events.json");

    cache.save_to_file("geo_cache.json")?;
    log::info!("💾 Cache saved to geo_cache.json");

    Ok(())
}

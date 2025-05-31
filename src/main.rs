mod sdk;
use sdk::{
    config::get_ors_api_key,
    departments::DepartmentLookup,
    events::{get_upcoming_events_by_region_and_month, filter_reachable_events},
    routing::cache::GeoCache,
    util::log::init_logging,
};

use std::{env, error::Error, fs::File, io::Write};

use serde_json;

fn main() -> Result<(), Box<dyn Error>> {
    init_logging();
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        log::warn!("Usage: {} <city> <department> <month> <year>", args[0]);
        std::process::exit(1);
    }

    let city = args[1].clone();
    let department = args[2].clone();
    let month: u32 = args[3].parse()?;
    let year: i32 = args[4].parse()?;

    let api_key = get_ors_api_key()?;
    let lookup = DepartmentLookup::from_csv("src/departments.csv")?;

    let origin = lookup.origin_from(&city, &department)
    .ok_or("Unknown origin department")?;

    let events = get_upcoming_events_by_region_and_month(month, year, &lookup)?;
    let mut cache = GeoCache::load_from_file("cache.json")?;
    let reachable_events = filter_reachable_events(&origin, &events, &lookup, &mut cache, &api_key, 2.0);
    cache.save_to_file("cache.json")?;

    log::info!("{} events are reachable within 2 hours.", reachable_events.len());

    let json = serde_json::to_string_pretty(&reachable_events)?;
    let mut file = File::create("reachable_events.json")?;
    file.write_all(json.as_bytes())?;
    log::info!("✅ Reachable events written to reachable_events.json");
    Ok(())
}

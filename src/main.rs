// main.rs (final example using the SDK)
mod sdk;
use sdk::routing::{cache::GeoCache, route::get_road_distance};
use sdk::{events::get_upcoming_events_by_region_and_month,config::get_ors_api_key};
use std::env;
use std::error::Error;
use sdk::departments::DepartmentLookup;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: {} <city> <department> <month> <year>", args[0]);
        std::process::exit(1);
    }

    let city = args[1].clone();
    let department = args[2].clone();
    let month: u32 = args[3].parse()?;
    let year: i32 = args[4].parse()?;

    let api_key = get_ors_api_key()?;
    let lookup = DepartmentLookup::from_csv("src/departments.csv")?;

    let origin = if let Some(department_name) = lookup.get_name(&department) {
        format!("{},{},France", city, department_name)
    } else {
        println!("Unknown origin department: {}", department);
        return Err("Unknown origin department".into());
    };

    let events = get_upcoming_events_by_region_and_month(month, year, &lookup)?;
    let mut reachable_events = Vec::new();

    // Initialize cache
    // let mut cache = GeoCache::default();
    let mut cache = GeoCache::load_from_file("cache.json")?;

    for event in events {
        if let Some(department_name) = lookup.get_name(&event.department) {
            let event_location = format!("{},{},France", department_name, event.location);
            if let Ok(summary) = get_road_distance(&origin, &event_location, &api_key, &mut cache) {
                if summary.duration_hours <= 2.0 {
                    println!("[REACHABLE] {} at {} ({:.1} km, {:.2} hrs, date={})",
                             event.title, event.location, summary.distance_km, summary.duration_hours, event.start_date);
                    reachable_events.push(event);
                } else {
                    // println!("[TOO FAR] {} at {} ({:.1} km, {:.2} hrs)",
                    //          event.title, event.location, summary.distance_km, summary.duration_hours);
                }
            } else {
                println!("[ERROR] Failed to calculate distance to event at {}", event.location);
            }
        } else {
            println!("Unknown department: {}", event.department);
        }
        // save the updated cache to the file
        cache.save_to_file("cache.json")?;
    }

    println!("\n{} events are reachable within 2 hours.", reachable_events.len());
    Ok(())
}

// main.rs (final example using the SDK)
mod sdk;
use sdk::{events::get_upcoming_events_by_region_and_month, routing::get_road_distance};
use std::env;
use std::error::Error;
use std::io;

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

    let api_key = env::var("ORS_API_KEY")?;
    let origin = format!("{}, France", city);
    let events = get_upcoming_events_by_region_and_month("", month, year)?;

    let mut reachable_events = Vec::new();

    for event in events {
        let event_location = format!("{}, France", event.location);

        if let Ok(summary) = get_road_distance(&origin, &event_location, &api_key) {
            if summary.duration_hours <= 2.0 {
                println!("[REACHABLE] {} at {} ({:.1} km, {:.2} hrs)",
                         event.title, event.location, summary.distance_km, summary.duration_hours);
                reachable_events.push(event);
            } else {
                println!("[TOO FAR] {} at {} ({:.1} km, {:.2} hrs)",
                         event.title, event.location, summary.distance_km, summary.duration_hours);
            }
        } else {
            println!("[ERROR] Failed to calculate distance to event at {}", event.location);
        }
    }

    println!("\n{} events are reachable within 2 hours.", reachable_events.len());

    Ok(())
}
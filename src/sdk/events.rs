// SDK for querying French Chess Federation events by region
use std::error::Error;

use chrono::{Datelike, Local, NaiveDate};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, USER_AGENT},
};
use scraper::{Html, Selector};
use serde::Serialize;

use super::{
    departments::DepartmentLookup,
    routing::{cache::GeoCache, route::get_road_distance},
};


#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub title: String,
    pub department: String,
    pub location: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub link: String,
}

fn parse_events_from_html(html: &str, month: u32, year: i32,lookup: &DepartmentLookup) -> Result<Vec<Event>, Box<dyn Error>> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("tr.liste_clair, tr.liste_fonce").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let mut events = Vec::new();

    for row in document.select(&row_selector) {
        let tds: Vec<_> = row.select(&td_selector).collect();

        if tds.len() < 5 {
            println!("[DEBUG] Skipping row due to insufficient columns: {} cols", tds.len());
            continue;
        }

        let title_elem = &tds[0];
        let title = title_elem.text().collect::<String>().trim().to_string();
        let location = tds[2].text().collect::<String>().trim().to_string();
        let department = tds[1].text().collect::<String>().trim().to_string();
        let start_date_str = tds[3].text().collect::<String>().trim().to_string();
        let end_date_str = tds[4].text().collect::<String>().trim().to_string();

        let start_date = NaiveDate::parse_from_str(&start_date_str, "%d/%m/%y").or_else(|_| NaiveDate::parse_from_str(&start_date_str, "%d/%m/%Y"));
        let end_date = NaiveDate::parse_from_str(&end_date_str, "%d/%m/%y").or_else(|_| NaiveDate::parse_from_str(&end_date_str, "%d/%m/%Y"));

        if let (Ok(start_date), Ok(end_date)) = (start_date, end_date) {
            if (start_date.month() != month && end_date.month() != month) || start_date.year() != year {
                println!("[DEBUG] Skipping due to date outside specified time: start={} (month {}), end={} (month {}), input month={}, input year={}", 
                    start_date, start_date.month(), end_date, end_date.month(), month, year);
                continue;
            }

            if !lookup.is_valid_department(&department) {
                println!("[DEBUG] Skipping event due to unknown department: {}", department);
                continue;
            }

            let link = title_elem.select(&a_selector)
                .next()
                .and_then(|a| a.value().attr("href"))
                .map(|href| format!("https://www.echecs.asso.fr/{}", href))
                .unwrap_or_default();

            events.push(Event {
                title,
                department,
                location,
                start_date,
                end_date,
                link,
            });
        } else {
            println!("[DEBUG] Failed to parse start or end date");
        }
    }

    Ok(events)
}


pub fn get_upcoming_events_by_region_and_month(month: u32, year: i32,lookup: &DepartmentLookup) -> Result<Vec<Event>, Box<dyn Error>> {
    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Mozilla/5.0".parse().unwrap());

    let today = Local::now().naive_local().date(); // get today's date
    let mut day = if today.year() == year && today.month() == month {
        today.day()
    } else {
        1
    };

    let mut events = Vec::new();
    println!("starting from the date={} month={}", day, month);

    loop {
        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => {
                let date_string = format!("{:02}/{:02}/{}", date.day(), date.month(), date.year());
                let url = format!("https://www.echecs.asso.fr/Calendrier.aspx?jour={}", date_string);
                let res = client.get(&url).headers(headers.clone()).send();
                match res {
                    Ok(response) => {
                        let html = response.text()?;

                        let mut daily_events = parse_events_from_html(&html, month, year,lookup)?;
                        events.append(&mut daily_events);
                    }
                    Err(err) => {
                        println!("[DEBUG] Failed to fetch {}: {}", url, err);
                    }
                }

                day += 1;
            }
            None => break, // Invalid date (e.g., April 31 or end of February)
        }
    }

    Ok(events)
}

pub fn filter_reachable_events(
    origin: &str,
    events: &[Event],
    lookup: &DepartmentLookup,
    cache: &mut GeoCache,
    ors_api_key: &str,
    max_hours: f64,
) -> Vec<Event> {
    let mut reachable = Vec::new();

    for event in events {
        if let Some(department_name) = lookup.get_name(&event.department) {
            let destination = format!("{},{},France", event.location, department_name);

            if origin.trim().eq_ignore_ascii_case(&destination.trim()) {
                println!(
                    "[REACHABLE - SAME LOCATION] {} at {} (0.0 km, 0.00 hrs, date={})",
                    event.title, event.location, event.start_date
                );
                reachable.push(event.clone());
                continue;
            }

            match get_road_distance(origin, &destination, ors_api_key, cache) {
                Ok(summary) if summary.duration_hours <= max_hours => {
                    println!(
                        "[REACHABLE] {} at {} ({:.1} km, {:.2} hrs, date={})",
                        event.title, event.location, summary.distance_km, summary.duration_hours, event.start_date
                    );
                    reachable.push(event.clone());
                }
                Ok(_) => {
                    // Too far
                    // println!("[TOO FAR] {}", event.title);
                }
                Err(err) => {
                    println!("[ERROR] Distance calc failed for {}: {}", event.title, err);
                }
            }
        } else {
            println!("[DEBUG] Unknown department: {}", event.department);
        }
    }

    reachable
}
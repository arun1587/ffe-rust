use chrono::{Datelike, NaiveDate};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, USER_AGENT},
};
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashSet;
use std::error::Error;

use super::departments::DepartmentLookup;
use super::routing::{cache::GeoCache, route::get_road_distance, service::RoutingProvider};

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash)]
pub struct Event {
    pub title: String,
    pub department: String,
    pub location: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub link: String,
}

/// Parses the MONTHLY CALENDAR view to find which days have events.
fn get_active_days_from_monthly_calendar(
    html: &str,
    month: u32,
    year: i32,
) -> Result<Vec<u32>, Box<dyn Error>> {
    let document = Html::parse_document(html);
    // Each calendar day is a `<td>`. We only care about those with an `onclick` event.
    let day_cell_selector = Selector::parse("td[onclick]").unwrap();
    // Inside a cell, the day number is in a link.
    let day_link_selector = Selector::parse("a.lien_texte").unwrap();
    // The presence of this paragraph indicates an event exists on that day.
    let event_marker_selector = Selector::parse("p.para_bleu_small").unwrap();

    let mut active_days = HashSet::new();

    for cell in document.select(&day_cell_selector) {
        // Only proceed if there's at least one event marker in the cell.
        if cell.select(&event_marker_selector).next().is_some() {
            if let Some(day_link) = cell.select(&day_link_selector).next() {
                // The link's href contains the full date, e.g., 'Calendrier.aspx?jour=14/06/2025'
                if let Some(href) = day_link.value().attr("href") {
                    if let Some(date_str) = href.split('=').last() {
                        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
                            // Only add the day if it's in the month we're targeting.
                            if date.month() == month && date.year() == year {
                                active_days.insert(date.day());
                            }
                        }
                    }
                }
            }
        }
    }

    let mut sorted_days: Vec<u32> = active_days.into_iter().collect();
    sorted_days.sort_unstable();
    Ok(sorted_days)
}

/// Parses the DETAILED LIST view for a single day.
fn parse_list_view_html(
    html: &str,
    lookup: &DepartmentLookup,
) -> Result<Vec<Event>, Box<dyn Error>> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("tr.liste_clair, tr.liste_fonce").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let mut events = Vec::new();

    for row in document.select(&row_selector) {
        let tds: Vec<_> = row.select(&td_selector).collect();
        if tds.len() < 5 {
            continue;
        }

        let title_elem = &tds[0];
        let title = title_elem.text().collect::<String>().trim().to_string();
        let location = tds[2].text().collect::<String>().trim().to_string();
        let department = tds[1].text().collect::<String>().trim().to_string();
        let start_date_str = tds[3].text().collect::<String>().trim().to_string();
        let end_date_str = tds[4].text().collect::<String>().trim().to_string();

        if !lookup.is_valid_department(&department) {
            continue;
        }

        if let (Ok(start_date), Ok(end_date)) = (
            NaiveDate::parse_from_str(&start_date_str, "%d/%m/%y")
                .or_else(|_| NaiveDate::parse_from_str(&start_date_str, "%d/%m/%Y")),
            NaiveDate::parse_from_str(&end_date_str, "%d/%m/%y")
                .or_else(|_| NaiveDate::parse_from_str(&end_date_str, "%d/%m/%Y")),
        ) {
            let link = title_elem
                .select(&a_selector)
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
        }
    }
    Ok(events)
}

pub fn get_events_for_month(
    month: u32,
    year: i32,
    client: &Client,
    lookup: &DepartmentLookup,
) -> Result<Vec<Event>, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Mozilla/5.0".parse().unwrap());

    // 1. Scout Mission: Get the monthly calendar view.
    let date_string = format!("01/{:02}/{}", month, year);
    let calendar_url = format!(
        "https://www.echecs.asso.fr/Calendrier.aspx?Date={}",
        date_string
    );
    log::info!("Scouting for active days from {}", calendar_url);
    let calendar_html = client
        .get(&calendar_url)
        .headers(headers.clone())
        .send()?
        .text()?;
    let active_days = get_active_days_from_monthly_calendar(&calendar_html, month, year)?;

    if active_days.is_empty() {
        log::info!("No events found in the calendar for {}/{}", month, year);
        return Ok(Vec::new());
    }
    log::info!(
        "Found {} active days. Fetching detailed event lists...",
        active_days.len()
    );

    // Use a HashSet to automatically handle duplicates (events spanning multiple days).
    let mut unique_events = HashSet::new();

    // 2. Targeted Strikes: Fetch details only for the active days.
    for day in active_days {
        let list_view_date = format!("{:02}/{:02}/{}", day, month, year);
        let list_view_url = format!(
            "https://www.echecs.asso.fr/Calendrier.aspx?jour={}",
            list_view_date
        );

        log::debug!("Fetching details from {}", list_view_url);
        let html = client
            .get(&list_view_url)
            .headers(headers.clone())
            .send()?
            .text()?;

        let daily_events = parse_list_view_html(&html, lookup)?;
        for event in daily_events {
            unique_events.insert(event);
        }
    }

    let mut final_events: Vec<Event> = unique_events.into_iter().collect();
    // Sort events by start date for a consistent output.
    final_events.sort_by_key(|e| e.start_date);

    Ok(final_events)
}

pub fn filter_reachable_events(
    origin_city: &str,
    origin: &str,
    events: &[Event],
    lookup: &DepartmentLookup,
    provider: &dyn RoutingProvider,
    cache: &mut GeoCache,
    max_hours: f64,
) -> Vec<Event> {
    let mut reachable = Vec::new();
    log::info!(
        "Filtering {} events for reachability from '{}' (max {:.2} hours)...",
        events.len(),
        origin,
        max_hours
    );

    for event in events {
        if origin_city
            .trim()
            .eq_ignore_ascii_case(&event.location.trim())
        {
            log::info!("[REACHABLE - SAME TOWN] {}", event.title);
            reachable.push(event.clone());
            continue;
        }

        if let Some(department_name) = lookup.get_name(&event.department) {
            let destination = format!("{}, {}", event.location, department_name);

            match get_road_distance(origin, &destination, provider, cache) {
                Ok(summary) if summary.duration_hours <= max_hours => {
                    log::info!(
                        "[REACHABLE] {} at {} ({:.1} km, {:.2} hrs)",
                        event.title,
                        event.location,
                        summary.distance_km,
                        summary.duration_hours
                    );
                    reachable.push(event.clone());
                }
                Ok(summary) => {
                    log::trace!(
                        "[TOO FAR] {} at {} ({:.2} hrs)",
                        event.title,
                        event.location,
                        summary.duration_hours
                    );
                }
                Err(err) => {
                    log::error!(
                        "Could not calculate route for '{}' to '{}': {}",
                        origin,
                        destination,
                        err
                    );
                }
            }
        }
    }
    reachable
}

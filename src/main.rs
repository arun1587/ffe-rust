// SDK for querying French Chess Federation events by region
use reqwest::blocking::Client;
use reqwest::header::{USER_AGENT, HeaderMap};
use scraper::{Html, Selector};
use chrono::{NaiveDate, Datelike};
use std::env;
use std::error::Error;


#[derive(Debug, Clone)]
pub struct Event {
    title: String,
    location: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    link: String,
}

fn parse_events_from_html(html: &str, region_query: &str, month: u32, year: i32) -> Result<Vec<Event>, Box<dyn Error>> {
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

            let link = title_elem.select(&a_selector)
                .next()
                .and_then(|a| a.value().attr("href"))
                .map(|href| format!("https://www.echecs.asso.fr/{}", href))
                .unwrap_or_default();

            if location.to_lowercase().contains(&region_query.to_lowercase()) {
                events.push(Event {
                    title,
                    location,
                    start_date,
                    end_date,
                    link,
                });
            } 
        } else {
            println!("[DEBUG] Failed to parse start or end date");
        }
    }

    Ok(events)
}

pub fn get_upcoming_events_by_region_and_month(region_query: &str, month: u32, year: i32) -> Result<Vec<Event>, Box<dyn Error>> {
    let date_string = format!("{:02}/{:02}/{}", 1, month, year);
    let url = format!("https://www.echecs.asso.fr/Calendrier.aspx?jour={}", date_string);

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36".parse()?
    );
    println!("Sending request to URL: {}", url);


    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    let response_raw = client.get(url).send()?;
    println!("[DEBUG] HTTP Status: {}", response_raw.status());

    let response = response_raw.text()?;

    parse_events_from_html(&response, region_query, month, year)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: ffe_cli <region> <month> <year>");
        std::process::exit(1);
    }

    let region_query = &args[1];
    let month: u32 = args[2].parse()?;
    let year: i32 = args[3].parse()?;

    let events = get_upcoming_events_by_region_and_month(region_query, month, year)?;

    for event in events {
        println!(
            "{} - {} | {} | {}
{}",
            event.start_date, event.end_date, event.title, event.location, event.link
        );
    }

    Ok(())
}

use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

use super::cache::{GeoCache, CityPairKey};
use super::geocode::geocode_city;

#[derive(Debug,Clone)]
pub struct RouteSummary {
    pub distance_km: f64,
    pub duration_hours: f64,
}

#[derive(Deserialize)]
struct Response {
    routes: Vec<Route>,
}

#[derive(Deserialize)]
struct Route {
    summary: Summary,
}

#[derive(Deserialize)]
struct Summary {
    distance: f64,
    duration: f64,
}

pub fn get_road_distance(city1: &str, city2: &str, api_key: &str,cache: &mut GeoCache) -> Result<RouteSummary, Box<dyn Error>> {
    let key = CityPairKey {
        origin: city1.to_string(),
        destination: city2.to_string(),
    };

    if let Some(&(dist, dur)) = cache.routes.get(&key) {
        println!("[CACHE HIT] {} → {}", city1, city2);
        return Ok(RouteSummary {
            distance_km: dist,
            duration_hours: dur,
        });
    }

    let (lon1, lat1) = match geocode_city(city1, api_key, cache) {
        Ok(coords) => coords,
        Err(e) => {
            println!("[ERROR] Failed to geocode city1 '{}': {}", city1, e);
            return Err(e);
        }
    };

    let (lon2, lat2) = match geocode_city(city2, api_key, cache) {
        Ok(coords) => coords,
        Err(e) => {
            println!("[ERROR] Failed to geocode city2 '{}': {}", city2, e);
            return Err(e);
        }
    };

    let url = "https://api.openrouteservice.org/v2/directions/driving-car";
    let body = serde_json::json!({
        "coordinates": [
            [lon1, lat1],
            [lon2, lat2]
        ]
    });

    println!("[REQUEST] POST {}", url);
    println!("[BODY] {}", body.to_string());

    let client = Client::new();
    let res = client
        .post(url)
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send();

    let res = match res {
        Ok(resp) => resp,
        Err(e) => {
            println!("[ERROR] Failed to send request to OpenRouteService: {}", e);
            return Err(Box::new(e));
        }
    };

    let text = match res.text() {
        Ok(t) => t,
        Err(e) => {
            println!("[ERROR] Failed to read response text: {}", e);
            return Err(Box::new(e));
        }
    };

    // Check if it's an error response
    if text.contains("\"error\"") {
        println!(
            "[ERROR] Routing failed for cities: {} → {}\n[RESPONSE] {}",
            city1, city2, text
        );
        return Err(format!("Routing failed for {} → {}", city1, city2).into());
    }

    // Try parsing the response into expected struct
    let route: Response = match serde_json::from_str(&text) {
        Ok(r) => r,
        Err(e) => {
            println!(
                "[ERROR] Failed to parse JSON response for {} → {}: {}",
                city1, city2, e
            );
            return Err(Box::new(e));
        }
    };

    let summary: &Summary = &route.routes[0].summary;

    let result = RouteSummary {
        distance_km: summary.distance / 1000.0,
        duration_hours: summary.duration / 3600.0,
    };

    cache.routes.insert(key, (result.distance_km, result.duration_hours));
   
    Ok(result)
}

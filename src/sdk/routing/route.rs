use serde::Deserialize;
use std::error::Error;
use futures::executor::block_on;
use crate::sdk::routing::cache::{GeoCache, CityPairKey};
use crate::sdk::util::rate_limit::Limiter;
use super::geocode::geocode_city;
use crate::sdk::routing::geocode::get_routable_coordinates;

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

pub fn get_road_distance(city1: &str, city2: &str, api_key: &str,cache: &mut GeoCache,limiter: &Limiter) -> Result<RouteSummary, Box<dyn Error>> {
    let key = CityPairKey {
        origin: city1.to_string(),
        destination: city2.to_string(),
    };

    if let Some(&(dist, dur)) = cache.routes.get(&key) {
        log::debug!("[CACHE HIT] {} → {}", city1, city2);
        return Ok(RouteSummary {
            distance_km: dist,
            duration_hours: dur,
        });
    }

    // wait to avoid rate limitation from ors
    block_on(limiter.until_ready());

    let (mut lon1, mut lat1) = geocode_city(city1, api_key, cache, limiter)?;
    let (mut lon2, mut lat2) = geocode_city(city2, api_key, cache, limiter)?;

    let url = "https://api.openrouteservice.org/v2/directions/driving-car";
    let client = reqwest::blocking::Client::new();


    let mut attempt = 0;
    let max_attempts = 2;

    while attempt < max_attempts {
        let body = serde_json::json!({
            "coordinates": [
                [lon1, lat1],
                [lon2, lat2]
            ]
        });

        let res = client
            .post(url)
            .header("Authorization", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send();

        let res = match res {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("Failed to send request to OpenRouteService: url={}, body={}: {}", url, body.to_string(), e);
                return Err(Box::new(e));
            }
        };

        let text = match res.text() {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to read response text: {}", e);
                return Err(Box::new(e));
            }
        };

        if !text.contains("\"error\"") {
            let route: Response = serde_json::from_str(&text)?;
            let summary: &Summary = &route.routes[0].summary;

            let result = RouteSummary {
                distance_km: summary.distance / 1000.0,
                duration_hours: summary.duration / 3600.0,
            };

            cache.routes.insert(key, (result.distance_km, result.duration_hours));
            return Ok(result);
        }

        if attempt == 0 && text.contains("Could not find routable point") {
            log::warn!("Unroutable point detected. Trying snapped coordinates via reverse geocoding
            cities: {} → {}\n
            lon1={} lat1={} lon2={} lat2={}",city1, city2, lon1, lat1,lon2,lat2);
            (lon1, lat1) = get_routable_coordinates(lon1, lat1, api_key)?;
            (lon2, lat2) = get_routable_coordinates(lon2, lat2, api_key)?;
        } else {
            log::error!(
                "Routing failed for cities: {} → {}\n[RESPONSE] {}\n [Cord] [{},{}] → [{},{}]",
                city1, city2, text, lon1, lat1, lon2, lat2
            );
            return Err(format!("Routing failed for {} → {}", city1, city2).into());
        }

        attempt += 1;
    }

    Err("Routing failed after retry with snapped coordinates.".into())
}

use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;
use super::cache::GeoCache;
use super::cache::CityPairKey;
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
        return Ok(RouteSummary {
            distance_km: dist,
            duration_hours: dur,
        });
    }

    let (lon1, lat1) = geocode_city(&city1, api_key,cache)?;
    let (lon2, lat2) = geocode_city(&city2, api_key,cache)?;

    let url = "https://api.openrouteservice.org/v2/directions/driving-car";
    let body = serde_json::json!({
        "coordinates": [
            [lon1, lat1],
            [lon2, lat2]
        ]
    });

    let client = Client::new();
    let res = client
        .post(url)
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?;

    let route: Response = res.json()?;
    let summary: &Summary = &route.routes[0].summary;

    let result = RouteSummary {
        distance_km: summary.distance / 1000.0,
        duration_hours: summary.duration / 3600.0,
    };

    cache.routes.insert(key, (result.distance_km, result.duration_hours));
   
    Ok(result)
}

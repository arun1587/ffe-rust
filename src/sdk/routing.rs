use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;
use std::collections::HashMap;

#[derive(Debug,Clone)]
pub struct RouteSummary {
    pub distance_km: f64,
    pub duration_hours: f64,
}

#[derive(Debug, Deserialize)]
struct GeoResponse {
    features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    geometry: Geometry,
}

#[derive(Debug, Deserialize)]
struct Geometry {
    coordinates: [f64; 2],
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

type Coord = (f64, f64);

#[derive(Default)]
pub struct GeoCache {
    geocode_cache: HashMap<String, Coord>,
    route_cache: HashMap<(String, String), RouteSummary>,
}


pub fn get_road_distance(city1: &str, city2: &str, api_key: &str,cache: &mut GeoCache) -> Result<RouteSummary, Box<dyn Error>> {
    let key = (city1.to_string(), city2.to_string());
    if let Some(summary) = cache.route_cache.get(&key) {
        return Ok(summary.clone());
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

    cache.route_cache.insert(key, result.clone());
    Ok(result)
}

fn geocode_city(city: &str, api_key: &str,cache: &mut GeoCache) -> Result<Coord, Box<dyn Error>> {
    if let Some(coord) = cache.geocode_cache.get(city) {
        return Ok(*coord);
    }

    let url = format!(
        "https://api.openrouteservice.org/geocode/search?api_key={}&text={}",
        api_key,
        city
    );

    let response = reqwest::blocking::get(&url);
    let response = match response {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("[ERROR] Failed to send geocoding request: {}", err);
            return Err(Box::new(err));
        }
    };

    let geo = response.json::<GeoResponse>();
    let geo = match geo {
        Ok(g) => g,
        Err(err) => {
            eprintln!("[ERROR] Failed to parse geocode response: {}", err);
            return Err(Box::new(err));
        }
    };
    let coords = geo.features.first().ok_or("No results from geocode")?.geometry.coordinates.clone();
    let coord = (coords[0], coords[1]);
    cache.geocode_cache.insert(city.to_string(), coord);
    Ok(coord)
}

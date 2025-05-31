use std::error::Error;
use serde::Deserialize;
use super::cache::GeoCache;
use crate::sdk::util::rate_limit::Limiter;
use futures::executor::block_on;
use serde_json::json;
use reqwest::blocking::Client;

#[derive(Deserialize)]
struct GeoResponse {
    features: Vec<Feature>,
}

#[derive(Deserialize)]
struct Feature {
    geometry: Geometry,
}

#[derive(Deserialize)]
struct Geometry {
    coordinates: [f64; 2],
}

type Coord = (f64, f64);

pub fn geocode_city(city: &str, api_key: &str,cache: &mut GeoCache,limiter: &Limiter) -> Result<Coord, Box<dyn Error>> {
    if let Some(coord) = cache.geocodes.get(city) {
        return Ok(*coord);
    }

    block_on(limiter.until_ready());

    let url = format!(
        "https://api.openrouteservice.org/geocode/search?api_key={}&text={}",
        api_key,
        city
    );

    let response = reqwest::blocking::get(&url);
    let response = match response {
        Ok(resp) => resp,
        Err(err) => {
            log::error!("Failed to send geocoding request for city {}: {}", city, err);
            return Err(Box::new(err));
        }
    };

    let geo = response.json::<GeoResponse>();
    let geo = match geo {
        Ok(g) => g,
        Err(err) => {
            log::error!("Failed to parse geocode response for city {}: {}",city, err);
            return Err(Box::new(err));
        }
    };
    let coords = geo.features.first().ok_or("No results from geocode")?.geometry.coordinates.clone();
    let coord = (coords[0], coords[1]);
    cache.geocodes.insert(city.to_string(), coord);
    Ok(coord)
}

pub fn get_routable_coordinates(lon: f64, lat: f64, api_key: &str) -> Result<(f64, f64), Box<dyn Error>> {
    let url = format!(
        "https://api.openrouteservice.org/geocode/reverse?point.lon={}&point.lat={}&api_key={}",
        lon, lat, api_key
    );

    let resp = reqwest::blocking::get(&url)?;
    let body: serde_json::Value = resp.json()?;

    let features = body["features"].as_array().ok_or("Invalid features array")?;

    for feature in features {
        if let Some(coords) = feature["geometry"]["coordinates"].as_array() {
            if coords.len() == 2 {
                let new_lon = coords[0].as_f64().unwrap_or(lon);
                let new_lat = coords[1].as_f64().unwrap_or(lat);

                // ✅ Check if coordinate is routable
                if is_routable((new_lon, new_lat), api_key) {
                    return Ok((new_lon, new_lat));
                }
            }
        }
    }

    Err("No routable point found".into())
}


fn is_routable(coord: (f64, f64), api_key: &str) -> bool {
    let client = Client::new();
    let url = "https://api.openrouteservice.org/v2/directions/driving-car";

    let body = json!({
        "coordinates": [
            [coord.0, coord.1],
            [coord.0, coord.1] // from and to the same point
        ]
    });

    let res = client
        .post(url)
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send();

    match res {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

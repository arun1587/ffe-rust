use super::cache::GeoCache;
use crate::sdk::util::rate_limit::GEOCODE_LIMITER;
use crate::sdk::util::rate_limit::DIRECTIONS_LIMITER;
use reqwest::Client;
use serde::Deserialize;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
pub struct GeoResponse {
    pub features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
pub struct Feature {
    pub geometry: Geometry,
}

#[derive(Debug, Deserialize)]
pub struct Geometry {
    pub coordinates: Vec<f64>,
}

pub type Coord = (f64, f64);

pub async fn geocode_city(city: &str, api_key: &str, cache: &mut GeoCache, client: &Client) -> Result<Coord> {
    if let Some(coord) = cache.geocodes.get(city) {
        return Ok(*coord);
    }
    log::info!("Waiting for geocode limiter before making API cal geocode_city...");

    GEOCODE_LIMITER.until_ready().await;

    let url = format!(
        "https://api.openrouteservice.org/geocode/search?api_key={}&text={}",
        api_key, city
    );

    let response = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to send geocoding request for city {}", city))?;

    let geo: GeoResponse = response
        .json()
        .await
        .with_context(|| format!("Failed to parse geocode response for city {}", city))?;

    let coords = geo.features.first()
        .context("No results from geocode")?
        .geometry.coordinates.clone();

    let coord = (coords[0], coords[1]);
    cache.geocodes.insert(city.to_string(), coord);
    Ok(coord)
}

pub async fn get_routable_coordinates(lon: f64, lat: f64, api_key: &str, client: &Client) -> Result<(f64, f64)> {
    let url = format!(
        "https://api.openrouteservice.org/geocode/reverse?point.lon={}&point.lat={}&api_key={}",
        lon, lat, api_key
    );
    log::info!("Waiting for geocode limiter before making API cal get_routable_coordinates...");

    GEOCODE_LIMITER.until_ready().await;

    let resp = client.get(&url).send().await?;
    let body: serde_json::Value = resp.json().await?;

    let features = body["features"].as_array().context("Invalid features array")?;

    for feature in features {
        if let Some(coords) = feature["geometry"]["coordinates"].as_array() {
            if coords.len() == 2 {
                let new_lon = coords[0].as_f64().unwrap_or(lon);
                let new_lat = coords[1].as_f64().unwrap_or(lat);
                if is_routable((new_lon, new_lat), api_key, client).await {
                    return Ok((new_lon, new_lat));
                }
            }
        }
    }

    Err(anyhow::anyhow!("No routable point found"))
}

pub async fn is_routable(coord: (f64, f64), api_key: &str, client: &Client) -> bool {
    let url = "https://api.openrouteservice.org/v2/directions/driving-car";
    let body = serde_json::json!({
        "coordinates": [
            [coord.0, coord.1],
            [coord.0, coord.1]
        ]
    });

    log::info!("Waiting for directions limiter before making API cal is_routable...");
    DIRECTIONS_LIMITER.until_ready().await;

    let res = client
        .post(url)
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match res {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

use std::error::Error;
use serde::Deserialize;
use super::cache::GeoCache;
use crate::sdk::util::rate_limit::Limiter;
use futures::executor::block_on;

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

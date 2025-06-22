use super::cache::{Coord, GeoCache};
use super::service::RoutingProvider;
use std::error::Error;

/// Gets coordinates for a city, using a cache to avoid redundant API calls.
pub fn get_or_cache_geocode(
    city: &str,
    provider: &dyn RoutingProvider,
    cache: &mut GeoCache,
) -> Result<Coord, Box<dyn Error>> {
    if let Some(coord) = cache.get_geocode(city) {
        log::debug!("Cache hit for geocode: {}", city);
        return Ok(coord);
    }

    log::debug!("Cache miss for geocode: {}. Calling provider.", city);
    let coord = provider.geocode(city)?;
    cache.insert_geocode(city, coord);
    Ok(coord)
}

/// Finds the nearest routable coordinate to a given point.
pub fn find_routable_coordinates(
    lon: f64,
    lat: f64,
    provider: &dyn RoutingProvider,
) -> Result<Coord, Box<dyn Error>> {
    let candidate_coords = provider.reverse_geocode((lon, lat))?;
    for coord in candidate_coords {
        if provider.is_routable(coord)? {
            log::info!(
                "Found routable coordinate for ({}, {}): {:?}",
                lon,
                lat,
                coord
            );
            return Ok(coord);
        }
    }
    Err("No routable point found among candidates".into())
}

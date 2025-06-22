use super::cache::{CityPairKey, GeoCache};
use super::error::RoutingError;
use super::geocode::{find_routable_coordinates, get_or_cache_geocode};
use super::service::RoutingProvider;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RouteSummary {
    pub distance_km: f64,
    pub duration_hours: f64,
}

/// Calculates road distance, handling caching and retrying with routable coordinates if necessary.
pub fn get_road_distance(
    city1: &str,
    city2: &str,
    provider: &dyn RoutingProvider,
    cache: &mut GeoCache,
) -> Result<RouteSummary, Box<dyn Error>> {
    let key = CityPairKey::new(city1, city2);
    if let Some(summary) = cache.get_route(&key) {
        log::debug!("[CACHE HIT] Route {} -> {}", city1, city2);
        return Ok(summary);
    }
    log::debug!("[CACHE MISS] Route {} -> {}", city1, city2);

    let mut coord1 = get_or_cache_geocode(city1, provider, cache)?;
    let mut coord2 = get_or_cache_geocode(city2, provider, cache)?;

    match provider.get_directions(coord1, coord2) {
        Ok(summary) => {
            cache.insert_route(key, summary);
            Ok(summary)
        }
        Err(e) => {
            if let Some(RoutingError::UnroutablePoint) = e.downcast_ref::<RoutingError>() {
                log::warn!(
                    "Unroutable point for {} -> {}. Finding snapped coordinates.",
                    city1,
                    city2
                );
                coord1 = find_routable_coordinates(coord1.0, coord1.1, provider)?;
                coord2 = find_routable_coordinates(coord2.0, coord2.1, provider)?;
                log::info!(
                    "Retrying with new coordinates: {:?} -> {:?}",
                    coord1,
                    coord2
                );

                let summary = provider.get_directions(coord1, coord2)?;
                cache.insert_route(key, summary);
                Ok(summary)
            } else {
                Err(e)
            }
        }
    }
}
